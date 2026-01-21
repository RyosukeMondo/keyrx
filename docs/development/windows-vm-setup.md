# Windows Development Environment (Vagrant)

This project includes a pre-configured Vagrant environment for Windows development and testing on Linux hosts using KVM/libvirt.

## Prerequisites

### Required Software

- **Linux Host** (tested on Ubuntu 22.04+)
- **KVM/libvirt** installed and running
- **Vagrant** 2.4+ (`sudo apt install vagrant` or from website)
- **vagrant-libvirt** plugin (`vagrant plugin install vagrant-libvirt`)

### Permissions Setup (One-Time)

Add yourself to the libvirt group:

```bash
sudo usermod -aG libvirt $USER
```

**IMPORTANT**: Log out and log back in for group changes to take effect.

Verify:
```bash
groups | grep libvirt  # Should show 'libvirt'
vagrant --version
vagrant plugin list | grep libvirt
```

## Quick Start

### Automated Method (Recommended)

From project root:

```bash
# Run tests on Windows
./scripts/platform/windows/test_vm.sh

# Run tests + UAT
./scripts/platform/windows/test_vm.sh --uat
```

The script automatically:
- Checks VM status (starts if needed)
- Syncs project files to VM
- Runs Windows-specific tests

### Manual Method

```bash
cd vagrant/windows

# First time: provision VM (10-20 minutes)
vagrant up

# Create snapshot after provisioning (recommended)
vagrant snapshot save provisioned

# SSH into VM
vagrant ssh

# Inside VM: run tests
cd C:\vagrant_project
cargo test -p keyrx_daemon --features windows
```

## VM Specifications

- **OS**: Windows 10 Enterprise Evaluation
- **VM Name**: keyrx2-win-test
- **Resources**: 4 CPU cores, 16GB RAM
- **Features**: Nested virtualization, USB tablet (mouse sync), optimized disk I/O
- **Tools**: Chocolatey, Git, Rust (stable + x86_64-pc-windows-msvc), rsync

## Access Methods

**SSH** (recommended):
```bash
cd vagrant/windows
vagrant ssh
```

**RDP** (for GUI testing):
- Host: `localhost:13389` or `<your-ip>:13389`
- Credentials: `vagrant` / `vagrant`

**GUI via virt-manager**:
```bash
virt-manager  # Find "keyrx2-win-test" VM
```

## Development Workflow

### Standard Workflow

1. **Edit code on Linux host**
2. **Sync to VM**:
   ```bash
   cd vagrant/windows
   vagrant rsync
   ```
3. **Test in VM**:
   ```bash
   vagrant ssh
   cd C:\vagrant_project
   cargo test -p keyrx_daemon --features windows
   ```
4. **Commit from Linux host**

### Quick Workflow (Using Script)

```bash
# From project root
./scripts/platform/windows/test_vm.sh
```

## File Syncing

The keyrx2 project root syncs to `C:\vagrant_project` in the VM.

**Excluded directories** (for performance):
- `.git/`, `node_modules/`, `vagrant/`
- `target/`, `build/`, `coverage/`, `logs/`
- `.spec-workflow/`

**Manual sync**:
```bash
cd vagrant/windows
vagrant rsync
```

**Verify sync** (inside VM):
```powershell
Test-Path C:\vagrant_project\Cargo.toml  # Should return True
```

## Snapshots

Snapshots save the entire VM state for instant restoration.

```bash
cd vagrant/windows

# Save after initial provisioning
vagrant snapshot save provisioned

# Save before risky operations
vagrant snapshot save before-test

# Restore to clean state
vagrant snapshot restore provisioned

# List all snapshots
vagrant snapshot list

# Delete snapshot
vagrant snapshot delete snapshot-name
```

## Testing Commands

### Inside VM (SSH session)

```powershell
cd C:\vagrant_project

# Windows-specific tests
cargo test -p keyrx_daemon --features windows

# Full workspace tests
cargo test --workspace

# Build release binary
cargo build --release

# Run daemon
.\target\release\keyrx_daemon.exe

# Run UAT script (if exists)
.\scripts\windows\UAT.ps1
```

### From Host (via script)

```bash
# Basic tests
./scripts/platform/windows/test_vm.sh

# With UAT
./scripts/platform/windows/test_vm.sh --uat
```

## VM Management

```bash
cd vagrant/windows

# Start VM
vagrant up

# Stop VM gracefully
vagrant halt

# Restart VM (applies Vagrantfile changes)
vagrant reload

# Destroy VM completely
vagrant destroy

# Check status
vagrant status

# Re-run provisioning
vagrant provision
```

## Port Forwarding

- **3389 → 13389** (RDP) - Remote Desktop
- **3030 → 3030** (localhost) - keyrx_daemon web server

Access the daemon web UI from your Linux host:
```bash
# Start daemon in VM, then from Linux:
firefox http://localhost:3030
```

## Troubleshooting

### VM won't start

```bash
# Check status
vagrant status
virsh list --all

# View detailed logs
vagrant up --debug

# Check libvirt daemon
systemctl status libvirtd
```

### Permission errors

```bash
# Verify group membership
groups | grep libvirt

# If missing, add yourself
sudo usermod -aG libvirt $USER
# Then log out and log back in

# Check socket permissions
ls -la /var/run/libvirt/libvirt-sock
```

### Sync not working

```bash
# Force sync
vagrant rsync

# Dry run (show what would sync)
vagrant rsync --dry-run

# Check inside VM
vagrant ssh
ls C:\vagrant_project
```

### Build errors in VM

```bash
# Inside VM, check Rust
rustc --version
cargo --version
rustup show

# Reinstall Windows target
rustup target add x86_64-pc-windows-msvc

# Update Rust
rustup update stable
```

### Network conflicts

If your LAN uses `192.168.121.x` subnet, edit `vagrant/windows/Vagrantfile`:

```ruby
# Uncomment this line:
libvirt.management_network_address = "10.10.10.0/24"
```

Then reload:
```bash
vagrant reload
```

### Out of disk space

```bash
# Check host disk space
df -h /var/lib/libvirt/images

# Remove old boxes
vagrant box prune

# List all VMs
virsh list --all

# Remove unused volumes
virsh vol-list default
virsh vol-delete <volume-name> default
```

## Performance Optimization

1. **Use snapshots** - Avoid re-provisioning by snapshotting after setup
2. **Keep VM running** - Use `halt` instead of `destroy`
3. **Selective syncing** - Only run `vagrant rsync` when needed
4. **Use SSH over GUI** - GUI (virt-manager) uses more resources
5. **Close unused apps** - In VM, close unnecessary Windows applications

## Integration with CI/CD

Test locally before pushing:

```bash
# Run Windows tests locally
./scripts/platform/windows/test_vm.sh

# If tests pass, commit
git add .
git commit -m "Fix Windows compatibility"
git push
```

## Advanced Usage

### Custom provisioning

Edit `vagrant/windows/Vagrantfile` to add tools:

```ruby
config.vm.provision "shell", privileged: true, name: "install-tools", inline: <<-SHELL
  choco install -y git rustup.install rsync cmake ninja
SHELL
```

Then re-provision:
```bash
vagrant provision --provision-with install-tools
```

### Increasing resources

Edit `vagrant/windows/Vagrantfile`:

```ruby
libvirt.cpus = 8        # More CPUs
libvirt.memory = 32768  # 32GB RAM
```

Then reload:
```bash
vagrant reload
```

## References

- Vagrant README: `vagrant/windows/README.md`
- Project docs: `/home/rmondo/repos/keyrx2/.claude/CLAUDE.md`
- Vagrant docs: https://developer.hashicorp.com/vagrant/docs
- vagrant-libvirt: https://vagrant-libvirt.github.io/vagrant-libvirt/
