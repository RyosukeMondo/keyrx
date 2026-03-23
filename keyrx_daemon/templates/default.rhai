// KeyRx Default Configuration — Dvorak Japanese IME Optimization
//
// Remaps low-frequency Dvorak bottom-row keys to high-frequency
// Japanese kana combos. Only active when Japanese IME is on.
//
// Based on 窓使いの憂鬱 (Mayu) DvorakY configuration.

device_start("*");

  // Dvorak-Japanese IME: remap bottom-row keys to kana combos
  // Active only when IME is on AND input language is Japanese
  // Physical keys S,D,F,G,H = Dvorak ;,Q,J,K,X
  when_start(["IME", "LANG_JA"]);
    sequence("VK_S", ["VK_Y", "VK_A"]);  // phys S (Dvk ;) -> ya (や)
    sequence("VK_D", ["VK_Y", "VK_O"]);  // phys D (Dvk Q) -> yo (よ)
    sequence("VK_F", ["VK_Y", "VK_E"]);  // phys F (Dvk J) -> ye (いぇ)
    sequence("VK_G", ["VK_Y", "VK_U"]);  // phys G (Dvk K) -> yu (ゆ)
    sequence("VK_H", ["VK_Y", "VK_I"]);  // phys H (Dvk X) -> yi (い)
  when_end();

device_end();
