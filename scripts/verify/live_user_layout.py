#!/usr/bin/env python3
"""Live proof of examples/user_layout.rhai through the REAL keyrx daemon.

Everything runs on a SCRATCH instance, harmless to the desktop and to the
user's real service:
  * scratch KEYRX_CONFIG_DIR, own port, short XDG_RUNTIME_DIR;
  * a uniquely named virtual uinput keyboard is the only device the profile
    (device_start("kx-ul-*"), made with `profiles import --device`) matches;
  * the daemon's output device is found by its exact name (keyrx-out-<pid>) and
    EVIOCGRABbed, so nothing reaches the desktop.

The oracle is independent of keyrx: the layout is parsed from the .rhai text
with regexes and key names are resolved to evdev codes through
/usr/include/linux/input-event-codes.h, so a wrong JIS code mapping, a layer
that does not engage or a stuck key all show up as a mismatch.

usage: live_user_layout.py [--bin DIR] [--only NAME[,NAME]] [--json OUT]
Needs /dev/uinput and /dev/input access (input group). Stop nothing else.
"""
import argparse, fcntl, glob, json, os, re, select, shutil, signal, statistics
import struct, subprocess, sys, tempfile, time

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
RHAI = os.path.join(REPO, "examples", "user_layout.rhai")
KBD_NAME = "kx-ul-test-kbd"
EVIOCGRAB = 0x40044590

# ---------------------------------------------------------------- key names
def header_codes():
    codes = {}
    for line in open("/usr/include/linux/input-event-codes.h"):
        m = re.match(r"#define\s+(KEY_[A-Z0-9_]+)\s+(\d+)\s*$", line.split("/*")[0].rstrip() + " ")
        if m:
            codes[m.group(1)] = int(m.group(2))
    return codes


H = header_codes()
ALIASES = {  # keyrx name -> linux KEY_ name (independent of keyrx's own tables)
    "LeftBracket": "LEFTBRACE", "RightBracket": "RIGHTBRACE", "Quote": "APOSTROPHE",
    "Period": "DOT", "Escape": "ESC", "Menu": "COMPOSE", "PrintScreen": "SYSRQ",
    # A JIS keyboard's 半角/全角 key is HID usage 0x35, which Linux reports as
    # KEY_GRAVE (KEY_ZENKAKUHANKAKU only comes from the rare HID Lang5 usage).
    "Zenkaku": "GRAVE", "全角": "GRAVE", "無変換": "MUHENKAN",
    "変換": "HENKAN", "Yen": "YEN", "Ro": "RO", "LMeta": "LEFTMETA", "LCtrl": "LEFTCTRL",
    "LShift": "LEFTSHIFT", "LAlt": "LEFTALT", "RCtrl": "RIGHTCTRL", "RShift": "RIGHTSHIFT",
    "RAlt": "RIGHTALT", "PageUp": "PAGEUP", "PageDown": "PAGEDOWN",
    "カタカナ": "KATAKANAHIRAGANA", "ひらがな": "HIRAGANA", "Backspace": "BACKSPACE",
}


def code(name):
    name = name[3:] if name.startswith("VK_") else name
    if re.fullmatch(r"Num\d", name):
        name = name[3:]
    key = "KEY_" + ALIASES.get(name, name).upper()
    return H[key]


NAMES = {v: k for k, v in H.items() if not k.startswith(("KEY_MAX", "KEY_CNT"))}

# ----------------------------------------------------------- layout oracle
class Layout:
    def __init__(self, text):
        self.holders = {}      # layer id -> [(src, tap or None)]
        self.layers = {}       # layer id -> {src: (mods, keys)}
        self.base = {}         # src -> ("map", mods, key) | ("tap", tap) | ("holdonly",)
        self.ime = []
        self.parse(text)

    @staticmethod
    def out(expr):
        m = re.fullmatch(r'\s*with_shift\("(\w+)"\)\s*', expr)
        if m:
            return ([code("VK_LShift")], [code(m.group(1))])
        m = re.fullmatch(r'\s*with_mods\("(\w+)",\s*(\w+),\s*(\w+),\s*(\w+),\s*(\w+)\)\s*', expr)
        if m:
            on = [a == "true" for a in m.groups()[1:]]
            mods = [c for f, c in zip(on, ("LShift", "LCtrl", "LAlt", "LMeta")) if f]
            return ([code("VK_" + c) for c in mods], [code(m.group(1))])
        m = re.fullmatch(r'\s*"([^"]+)"\s*', expr)
        return ([], [code(m.group(1))])

    @staticmethod
    def mid(name):
        return int(name[3:], 16)

    def parse(self, text):
        cur = None
        for raw in text.splitlines():
            line = raw.split("//")[0].strip()
            if not line:
                continue
            m = re.match(r'when_start\("(MD_\w+)"\);', line)
            if m:
                cur = self.mid(m.group(1)); self.layers.setdefault(cur, {}); continue
            if line.startswith("when_start("):
                cur = "ime"; continue
            if line.startswith("when_end"):
                cur = None; continue
            m = re.match(r'tap_hold\("(\w+)",\s*"(\w+)",\s*"(MD_\w+)",\s*\d+\);', line)
            if m:
                src, tap, lid = code(m.group(1)), code(m.group(2)), self.mid(m.group(3))
                self.holders.setdefault(lid, []).append((src, tap))
                self.base[src] = ("tap", tap)
                continue
            m = re.match(r'hold_only\("(\w+)",\s*"(MD_\w+)"\);', line)
            if m:
                src, lid = code(m.group(1)), self.mid(m.group(2))
                self.holders.setdefault(lid, []).append((src, None))
                self.base[src] = ("holdonly",)
                continue
            m = re.match(r'map\("([^"]+)",\s*(.+)\);\s*$', line)
            if m:
                src, (mods, keys) = code(m.group(1)), self.out(m.group(2))
                if cur == "ime":
                    continue
                if cur is None:
                    self.base[src] = ("map", mods, keys[0])
                else:
                    self.layers[cur][src] = (mods, keys[0])
                continue
            if line.startswith("sequence("):
                continue
            if line.startswith(("device_start", "device_end")):
                continue
            raise SystemExit(f"oracle cannot parse: {raw!r}")

    def base_out(self, src):
        """(mods, key) the base layer types for a quick tap of `src`, or None."""
        b = self.base.get(src)
        if b is None:
            return ([], src)          # passthrough
        if b[0] == "map":
            return (b[1], b[2])
        if b[0] == "tap":
            return ([], b[1])
        return None

# ----------------------------------------------------------------- rig
class Rig:
    def __init__(self, bindir, work):
        self.bin, self.work = bindir, work
        self.cfg = os.path.join(work, "cfg")
        self.xdg = tempfile.mkdtemp(prefix="kxul")
        self.port = 19000 + os.getpid() % 900
        self.env = dict(os.environ, KEYRX_CONFIG_DIR=self.cfg, XDG_RUNTIME_DIR=self.xdg,
                        KEYRX_PORT=str(self.port), RUST_LOG="info")
        self.daemon = None
        self.log = open(os.path.join(work, "daemon.log"), "w")

    # virtual keyboard ----------------------------------------------------
    def make_kbd(self):
        self.ufd = os.open("/dev/uinput", os.O_WRONLY | os.O_NONBLOCK)
        fcntl.ioctl(self.ufd, 0x40045564, 1)
        fcntl.ioctl(self.ufd, 0x40045564, 0)
        for k in range(1, 250):
            fcntl.ioctl(self.ufd, 0x40045565, k)
        os.write(self.ufd, struct.pack("80sHHHHi64i64i64i64i", KBD_NAME.encode(), 3, 0x1234,
                                       0x5678, 1, 0, *([0] * 256)))
        fcntl.ioctl(self.ufd, 0x5501)
        time.sleep(0.6)

    def kill_kbd(self):
        try:
            fcntl.ioctl(self.ufd, 0x5502)
        except Exception:
            pass

    def _w(self, t, c, v):
        os.write(self.ufd, struct.pack("<qqHHi", 0, 0, t, c, v))

    def key(self, c, v):
        self._w(1, c, v); self._w(0, 0, 0)

    # config ----------------------------------------------------------------
    def cli(self, *args, check=True):
        r = subprocess.run([f"{self.bin}/keyrx_daemon", *args], env=self.env, capture_output=True, text=True)
        if check and r.returncode:
            raise SystemExit(f"keyrx_daemon {' '.join(args)} failed: {r.stdout}{r.stderr}")
        return r

    def setup_profiles(self, rhai=RHAI):
        shutil.rmtree(self.cfg, ignore_errors=True)
        os.makedirs(self.cfg)
        self.cli("profiles", "import", rhai, "ul-test", "--device", "kx-ul-*", "--activate")
        swap = os.path.join(self.work, "swap.rhai")
        open(swap, "w").write('device_start("kx-ul-*");\nmap("F13", "VK_F14");\ndevice_end();\n')
        self.cli("profiles", "import", swap, "swap")

    # daemon ------------------------------------------------------------------
    def start(self, extra=()):
        self.daemon = subprocess.Popen([f"{self.bin}/keyrx_daemon", "run", *extra], env=self.env,
                                       stdout=self.log, stderr=subprocess.STDOUT)
        name = f"keyrx-out-{self.daemon.pid}"
        deadline = time.time() + 15
        self.out_fd = None
        while time.time() < deadline and self.out_fd is None:
            for n in glob.glob("/sys/class/input/event*/device/name"):
                try:
                    if open(n).read().strip() == name:
                        self.out_fd = os.open("/dev/input/" + n.split("/")[4], os.O_RDONLY | os.O_NONBLOCK)
                except OSError:
                    pass
            time.sleep(0.05)
        if self.out_fd is None:
            raise SystemExit("daemon output device never appeared; see daemon.log")
        fcntl.ioctl(self.out_fd, EVIOCGRAB, 1)
        time.sleep(0.8)
        self.drain()

    def stop(self):
        if self.daemon and self.daemon.poll() is None:
            self.daemon.send_signal(signal.SIGTERM)
            try:
                self.daemon.wait(5)
            except subprocess.TimeoutExpired:
                self.daemon.kill()
        if getattr(self, "out_fd", None) is not None:
            os.close(self.out_fd); self.out_fd = None

    def grabbed_inputs(self):
        names = set()
        for fd in glob.glob(f"/proc/{self.daemon.pid}/fd/*"):
            try:
                t = os.readlink(fd)
            except OSError:
                continue
            if t.startswith("/dev/input/event") and "keyrx" not in t:
                n = "/sys/class/input/" + t.split("/")[-1] + "/device/name"
                names.add(open(n).read().strip())
        return names

    # io ----------------------------------------------------------------------
    def drain(self):
        ev = []
        while select.select([self.out_fd], [], [], 0)[0]:
            try:
                d = os.read(self.out_fd, 24 * 64)
            except BlockingIOError:
                break
            for i in range(0, len(d), 24):
                s, us, t, c, v = struct.unpack("<qqHHi", d[i:i + 24])
                if t == 1 and v in (0, 1):
                    ev.append((s + us / 1e6, c, v))
        return ev

    def play(self, steps, settle=0.06):
        """steps: (delay_ms_before, 'p'|'r', code). Returns captured output events."""
        self.drain()
        out = []
        for dt, op, c in steps:
            if dt:
                end = time.perf_counter() + dt / 1000
                while time.perf_counter() < end:
                    out += self.drain()
                    time.sleep(0.0005)
            self.key(c, 1 if op == "p" else 0)
        end = time.perf_counter() + settle
        while time.perf_counter() < end:
            out += self.drain()
            time.sleep(0.001)
        return out

# ------------------------------------------------------------- checking
class Report:
    def __init__(self):
        self.rows = {}      # section -> [total, passed]
        self.fails = []

    def check(self, section, ok, msg):
        r = self.rows.setdefault(section, [0, 0])
        r[0] += 1
        r[1] += bool(ok)
        if not ok:
            self.fails.append(f"[{section}] {msg}")
        return ok


def nm(c):
    return NAMES.get(c, str(c)).replace("KEY_", "")


def fmt(ev):
    return " ".join(("+" if v else "-") + nm(c) for _, c, v in ev) or "(nothing)"


def net_held(ev):
    held = set()
    for _, c, v in ev:
        (held.add if v else held.discard)(c)
    return held


def presses(ev):
    return [c for _, c, v in ev if v == 1]


def stroke_ok(ev, mods, key):
    """presses == mods then key; mods down when key pressed; key up before mods; nothing stuck."""
    if presses(ev) != mods + [key]:
        return False
    held, ok = set(), True
    for _, c, v in ev:
        if v:
            if c == key and not all(m in held for m in mods):
                ok = False
            held.add(c)
        else:
            if c in mods and key in held:
                ok = False
            held.discard(c)
    return ok and not held

# ---------------------------------------------------------------- scenarios
def S(*steps):
    return list(steps)


def run_layers(rig, lay, rep):
    for lid in sorted(lay.layers):
        for hsrc, t in lay.holders[lid]:
            hn = nm(hsrc)
            for src, (mods, key) in sorted(lay.layers[lid].items()):
                if src in {h for hs in lay.holders.values() for h, _ in hs}:
                    continue  # holder keys inside a layer are reported separately
                tag = f"MD_{lid:02X}[{hn}] {nm(src)}->{'+'.join(nm(m) for m in mods + [key])}"
                cases = {
                    "layer.permissive": S((0, "p", hsrc), (50, "p", src), (40, "r", src), (10, "r", hsrc)),
                    "layer.hold>200ms": S((0, "p", hsrc), (260, "p", src), (40, "r", src), (10, "r", hsrc)),
                    # layer already active (held >200ms): releasing it first must not drop the key
                    "layer.rollover-late": S((0, "p", hsrc), (260, "p", src), (30, "r", hsrc), (40, "r", src)),
                }
                for sec, steps in cases.items():
                    ev = rig.play(steps)
                    rep.check(sec, stroke_ok(ev, mods, key), f"{tag}: got {fmt(ev)}")
                # Fast roll (<200ms, layer key released BEFORE the typed key) is a
                # tap by design (permissive hold): the tap output, then the base key.
                bmods, bkey = lay.base_out(src)
                tap = [t] if t else []
                ev = rig.play(S((0, "p", hsrc), (50, "p", src), (30, "r", hsrc), (40, "r", src)))
                rep.check("layer.rollover-early-is-tap", presses(ev) == tap + bmods + [bkey] and not net_held(ev),
                          f"{tag}: got {fmt(ev)} want tap={[nm(x) for x in tap]} then {[nm(x) for x in bmods + [bkey]]}")
                # same key twice, then release order: layer key last
                ev = rig.play(S((0, "p", hsrc), (50, "p", src), (30, "r", src), (30, "p", src),
                                (30, "r", src), (10, "r", hsrc)))
                want = (mods + [key]) * 2
                rep.check("layer.repeat", presses(ev) == want and not net_held(ev), f"{tag}: got {fmt(ev)}")


def run_fallthrough(rig, lay, rep):
    holders = {h for hs in lay.holders.values() for h, _ in hs}
    keys = sorted(set(lay.base) | {code(k) for k in
                  ("VK_Z", "VK_Space") })
    for lid in sorted(lay.holders):
        hsrc = lay.holders[lid][0][0]
        for src in keys:
            if src in holders or src in lay.layers.get(lid, {}):
                continue
            want = lay.base_out(src)
            if want is None:
                continue
            ev = rig.play(S((0, "p", hsrc), (50, "p", src), (40, "r", src), (10, "r", hsrc)))
            rep.check("layer.fallthrough", stroke_ok(ev, *want),
                      f"MD_{lid:02X} holding {nm(hsrc)}, unmapped {nm(src)}: got {fmt(ev)} want {want}")


def run_base(rig, lay, rep):
    for src in sorted(set(lay.base) | {code("VK_Z")}):
        b = lay.base.get(src)
        tag = f"base {nm(src)}"
        if b is None or b[0] == "map":
            mods, key = lay.base_out(src)
            ev = rig.play(S((0, "p", src), (30, "r", src)))
            rep.check("base.tap", stroke_ok(ev, mods, key), f"{tag}: got {fmt(ev)}")
        elif b[0] == "tap":
            ev = rig.play(S((0, "p", src), (50, "r", src)))
            rep.check("base.taphold-tap", stroke_ok(ev, [], b[1]), f"{tag} tap: got {fmt(ev)}")
            ev = rig.play(S((0, "p", src), (300, "r", src)))
            rep.check("base.taphold-hold-alone", ev == [], f"{tag} held alone: got {fmt(ev)}")
        else:
            ev = rig.play(S((0, "p", src), (50, "r", src)))
            rep.check("base.holdonly-tap-suppressed", ev == [], f"{tag} tap: got {fmt(ev)}")
            ev = rig.play(S((0, "p", src), (300, "r", src)))
            rep.check("base.holdonly-tap-suppressed", ev == [], f"{tag} long: got {fmt(ev)}")


def run_repeats(rig, lay, rep):
    for src in sorted(lay.base):
        b = lay.base[src]
        if b[0] != "map":
            continue
        ev = rig.play(S(*[(0 if i == 0 else 25, op, src) for i in range(4) for op in "pr"]))
        rep.check("base.same-key-repeat", presses(ev) == (b[1] + [b[2]]) * 4 and not net_held(ev),
                  f"{nm(src)} x4: got {fmt(ev)}")


def inverse_map(lay):
    """output key -> physical source typing it as a plain tap (no modifiers)."""
    inv = {}
    for k in range(1, 250):
        o = lay.base_out(k)
        if o and not o[0] and k not in {h for hs in lay.holders.values() for h, _ in hs}:
            inv.setdefault(o[1], k)
    return inv


SENTENCES = [
    "kyou wa ii tenki desu ne watashi no namae wa sensei to iimasu",
    "nihongo no nyuuryoku wo dvorak de tesuto shiteimasu aiueo kakikukeko",
    "tokyo to osaka no aida wo shinkansen de ikimasu domo arigato gozaimasu",
]


def run_typing(rig, lay, rep):
    inv = inverse_map(lay)
    for gap, hold, label in ((90, 35, "typing.sequential"), (45, 20, "typing.fast")):
        for sent in SENTENCES:
            wants, steps = [], []
            for ch in sent:
                if ch == " ":
                    k = code("VK_Space")
                    src = inv.get(k)
                else:
                    k = H["KEY_" + ch.upper()]
                    src = inv.get(k)
                if src is None:
                    continue
                wants.append(k)
                steps += [(gap, "p", src), (hold, "r", src)]
            ev = rig.play(steps, settle=0.15)
            rep.check(label, presses(ev) == wants and not net_held(ev) and
                      sum(1 for e in ev if e[2]) == sum(1 for e in ev if not e[2]),
                      f"{sent!r}: typed {[nm(c) for c in presses(ev)]} want {[nm(c) for c in wants]} held={net_held(ev)}")
    # Overlapped rollover (next key down before previous up), fixed seed, every
    # hold-capable key included: only the invariants are asserted.
    import random
    rnd = random.Random(7)
    keys = [k for k in range(1, 100) if k not in (1, 29, 97, 125)]
    for trial in range(40):
        seq, down, steps = [], [], []
        for _ in range(60):
            if down and (len(down) >= 3 or rnd.random() < 0.5):
                steps.append((rnd.randint(5, 40), "r", down.pop(rnd.randrange(len(down)))))
            else:
                k = rnd.choice([x for x in keys if x not in down])
                down.append(k); steps.append((rnd.randint(5, 40), "p", k))
        for k in down:
            steps.append((rnd.randint(5, 40), "r", k))
        ev = rig.play(steps, settle=0.5)
        rep.check("typing.chaos-net-zero", not net_held(ev), f"trial {trial}: stuck {[nm(c) for c in net_held(ev)]}")
        bal = {}
        for _, c, v in ev:
            bal[c] = bal.get(c, 0) + (1 if v else -1)
            if bal[c] < 0:
                break
        rep.check("typing.chaos-release-without-press", all(v >= 0 for v in bal.values()),
                  f"trial {trial}: unbalanced {bal}")


def run_min_key_down(rig, lay, rep):
    worst = 1e9
    for src in [code(n) for n in ("VK_Z", "VK_F2", "VK_Num3", "VK_Space")]:
        o = lay.base_out(src)
        for _ in range(8):
            ev = rig.play(S((0, "p", src), (1, "r", src)))  # 1 ms physical tap
            ts = {v: t for t, c, v in ev if c == o[1]}
            if 1 in ts and 0 in ts:
                worst = min(worst, (ts[0] - ts[1]) * 1000)
    rep.check("min-key-down", worst >= 4.0, f"shortest output key-down {worst:.2f} ms (want >= ~5)")
    return worst


def run_latency(rig, lay, rep):
    out = {}
    for label, src in (("simple", code("VK_Z")), ("remapped", code("VK_W"))):
        want = lay.base_out(src)[1]
        lats = []
        for _ in range(250):
            rig.drain()
            t0 = time.perf_counter()
            rig.key(src, 1)
            while True:
                if select.select([rig.out_fd], [], [], 0.2)[0]:
                    ev = rig.drain()
                    if any(c == want and v for _, c, v in ev):
                        lats.append((time.perf_counter() - t0) * 1000)
                        break
                else:
                    break
            rig.key(src, 0)
            time.sleep(0.02)
        lats.sort()
        pct = lambda p: lats[min(len(lats) - 1, int(len(lats) * p))]
        out[label] = {"n": len(lats), "p50": pct(0.5), "p95": pct(0.95), "p99": pct(0.99), "max": lats[-1]}
        rep.check("latency", len(lats) == 250 and pct(0.99) < 10, f"{label}: {out[label]}")
    return out


def run_reload(rig, lay, rep):
    """Layer key held while the profile is switched / the source is hot-reloaded."""
    holder = lay.holders[0][0][0]
    # 1. switch profile while B (MD_00) is held, then release; nothing may stay down.
    rig.drain()
    rig.key(holder, 1); time.sleep(0.3)
    rig.key(code("VK_Num2"), 1); time.sleep(0.05)           # MD_00 Num2 -> Left (held)
    rig.cli("profiles", "activate", "swap")
    time.sleep(1.0)
    rig.key(code("VK_Num2"), 0); rig.key(holder, 0); time.sleep(0.3)
    ev = rig.drain()
    rep.check("reload.profile-switch-held-layer-key", not net_held(ev), f"net held after switch: {fmt(ev)}")
    # the swapped profile is really live: F13 -> F14, and layout keys are plain again
    ev = rig.play(S((0, "p", code("VK_F13")), (30, "r", code("VK_F13"))))
    rep.check("reload.swap-profile-live", presses(ev) == [code("VK_F14")], f"F13 -> {fmt(ev)}")
    # 2. switch back and verify the full layer works again
    rig.cli("profiles", "activate", "ul-test")
    time.sleep(1.0)
    ev = rig.play(S((0, "p", holder), (260, "p", code("VK_Num2")), (40, "r", code("VK_Num2")), (10, "r", holder)))
    rep.check("reload.back-to-user-layout", presses(ev) == [code("VK_Left")] and not net_held(ev), fmt(ev))
    # 3. hot reload of the source (watcher) while a layer key is held
    path = os.path.join(rig.cfg, "profiles", "ul-test.rhai")
    src = open(path).read()
    rig.key(holder, 1); time.sleep(0.3)
    open(path, "w").write(src.replace("device_end();", 'map("F20", "VK_F19");\ndevice_end();'))
    time.sleep(4.0)
    rig.key(holder, 0); time.sleep(0.4)
    ev = rig.drain()
    rep.check("reload.hot-reload-held-layer-key", not net_held(ev), f"net held after hot reload: {fmt(ev)}")
    ev = rig.play(S((0, "p", code("VK_F20")), (30, "r", code("VK_F20"))))
    rep.check("reload.hot-reload-applied", presses(ev) == [code("VK_F19")], f"F20 -> {fmt(ev)}")
    ev = rig.play(S((0, "p", holder), (260, "p", code("VK_Num2")), (40, "r", code("VK_Num2")), (10, "r", holder)))
    rep.check("reload.layers-intact-after-hot-reload", presses(ev) == [code("VK_Left")] and not net_held(ev), fmt(ev))
    # 4. a layer key held across a BROKEN edit: running config kept
    rig.key(holder, 1); time.sleep(0.3)
    open(path, "w").write(src.replace("device_end();", 'map("F20", \n'))
    time.sleep(4.0)
    ev = rig.play(S((0, "p", code("VK_Num2")), (40, "r", code("VK_Num2"))))
    rig.key(holder, 0); time.sleep(0.2)
    rep.check("reload.broken-edit-keeps-running-config", presses(ev) == [code("VK_Left")],
              f"MD_00 Num2 after broken edit: {fmt(ev)}")
    open(path, "w").write(src)
    time.sleep(3.0)


def run_status(rig, rep):
    r = rig.cli("status", check=False)
    rep.check("status", r.returncode == 0 and "ul-test" in r.stdout + r.stderr, (r.stdout + r.stderr)[:300])
    g = rig.grabbed_inputs()
    rep.check("status.grabbed-only-test-device", g == {KBD_NAME}, f"grabbed {g}")


def run_emergency(rig, rep):
    """Raw-physical emergency chord with LCtrl remapped (tap=Space, hold=MD_05)."""
    rig.key(29, 1); time.sleep(0.05); rig.key(97, 1); time.sleep(0.05); rig.key(1, 1)
    t0 = time.time()
    while rig.daemon.poll() is None and time.time() - t0 < 6:
        time.sleep(0.05)
    died = rig.daemon.poll() is not None
    rep.check("emergency.chord-LCtrl+RCtrl+Esc", died, f"daemon exit={rig.daemon.poll()} after {time.time() - t0:.2f}s")
    for c in (1, 97, 29):
        rig.key(c, 0)
    return time.time() - t0


def run_emergency_hold(rig, rep):
    rig.key(1, 1)
    t0 = time.time()
    while rig.daemon.poll() is None and time.time() - t0 < 8:
        time.sleep(0.05)
    rep.check("emergency.hold-Esc-3s", rig.daemon.poll() is not None, f"exit={rig.daemon.poll()} after {time.time() - t0:.2f}s")
    rig.key(1, 0)

# -------------------------------------------------------------------- main
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bin", default=os.path.join(REPO, "target", "release"))
    ap.add_argument("--only", default="")
    ap.add_argument("--json", default="")
    a = ap.parse_args()
    only = set(filter(None, a.only.split(",")))
    lay = Layout(open(RHAI).read())
    work = tempfile.mkdtemp(prefix="kxul-work")
    rig, rep = Rig(a.bin, work), Report()
    extra = {}
    try:
        rig.make_kbd()
        rig.setup_profiles()
        rig.start()
        steps = [("status", lambda: run_status(rig, rep)),
                 ("base", lambda: run_base(rig, lay, rep)),
                 ("repeats", lambda: run_repeats(rig, lay, rep)),
                 ("layers", lambda: run_layers(rig, lay, rep)),
                 ("fallthrough", lambda: run_fallthrough(rig, lay, rep)),
                 ("typing", lambda: run_typing(rig, lay, rep)),
                 ("minkeydown", lambda: extra.__setitem__("min_key_down_ms", run_min_key_down(rig, lay, rep))),
                 ("latency", lambda: extra.__setitem__("latency_ms", run_latency(rig, lay, rep))),
                 ("reload", lambda: run_reload(rig, lay, rep))]
        for name, fn in steps:
            if not only or name in only:
                t = time.time(); fn(); print(f"{name}: {time.time() - t:.1f}s", flush=True)
        if not only or "emergency" in only:
            extra["emergency_chord_s"] = run_emergency(rig, rep)
            rig.stop()
            rig.start()
            run_emergency_hold(rig, rep)
    finally:
        rig.stop()
        rig.kill_kbd()
    print("\n%-44s %7s %7s" % ("scenario", "total", "passed"))
    for k, (t, p) in sorted(rep.rows.items()):
        print("%-44s %7d %7d %s" % (k, t, p, "" if t == p else "<-- FAIL"))
    tot = sum(t for t, _ in rep.rows.values()); ok = sum(p for _, p in rep.rows.values())
    print("%-44s %7d %7d" % ("TOTAL", tot, ok))
    print(json.dumps(extra, indent=1))
    for f in rep.fails[:25]:
        print("FAIL", f)
    if len(rep.fails) > 25:
        print(f"... {len(rep.fails) - 25} more failures (see --json)")
    if a.json:
        json.dump({"rows": rep.rows, "extra": extra, "fails": rep.fails}, open(a.json, "w"), indent=1)
    print("daemon log:", os.path.join(work, "daemon.log"))
    sys.exit(0 if tot == ok else 1)


if __name__ == "__main__":
    main()
