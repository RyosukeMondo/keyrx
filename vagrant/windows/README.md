# keyrx2 Windows Testing VM

Windows 10 VM for testing keyrx2 daemon on Windows using Vagrant + KVM/libvirt.

## Quick Start

```bash
cd /home/rmondo/repos/keyrx2/vagrant/windows

# First time setup (10-20 min)
vagrant up

# SSH into VM
vagrant ssh

# Sync files after changes
vagrant rsync

# Stop VM
vagrant halt
```

## Running keyrx2 Tests on Windows

### From Host (Automated)

Use the provided script from project root:

```bash
cd /home/rmondo/repos/keyrx2
./scripts/platform/windows/test_vm.sh

# Run with UAT
./scripts/platform/windows/test_vm.sh --uat
```

### From Inside VM (Manual)

```bash
# SSH into VM
vagrant ssh

# Navigate to project
cd C:\vagrant_project

# Run Windows-specific tests
cargo test -p keyrx_daemon --features windows

# Run full workspace tests
cargo test --workspace

# Build release binary
cargo build --release

# Run daemon
.\target\release\keyrx_daemon.exe
```

## VM Specifications

- **OS**: Windows 10 Enterprise Evaluation
- **VM Name**: keyrx2-win-test
- **CPU**: 4 cores (host-passthrough)
- **RAM**: 16GB
- **Sync**: `keyrx2/` → `C:\vagrant_project`
- **Tools**: Git, Rust (stable + x86_64-pc-windows-msvc), rsync

## Port Forwarding

- **3389 → 13389** (RDP) - Remote Desktop access
- **3030 → 3030** (localhost) - keyrx_daemon web server

## Access Methods

**SSH** (recommended for development):
```bash
vagrant ssh
```

**RDP** (for GUI testing):
- Host: `localhost:13389` or `<your-ip>:13389`
- User: `vagrant`
- Password: `vagrant`

**GUI via virt-manager**:
```bash
virt-manager  # Find "keyrx2-win-test" VM
```

## File Syncing

The entire keyrx2 project is synced to `C:\vagrant_project` in the VM.

**Sync workflow**:
1. Edit code on Linux host
2. Run `vagrant rsync` to push changes to VM
3. Test in VM
4. Commit from Linux host

**Excluded from sync** (for performance):
- `.git/`
- `node_modules/`
- `vagrant/`
- `target/`
- `build/`
- `coverage/`
- `logs/`
- `.spec-workflow/`

## Snapshots (Recommended)

Create a snapshot after initial provisioning to save time:

```bash
# After first successful 'vagrant up'
vagrant snapshot save provisioned

# Restore clean state anytime
vagrant snapshot restore provisioned

# Before running destructive tests
vagrant snapshot save before-test
vagrant snapshot restore before-test

# List snapshots
vagrant snapshot list

# Delete snapshot
vagrant snapshot delete snapshot-name
```

## Troubleshooting

**VM won't start**:
```bash
vagrant status
vagrant up --debug
virsh list --all
```

**Sync issues**:
```bash
# Force sync
vagrant rsync

# Check what will be synced
vagrant rsync --dry-run
```

**Build errors**:
```bash
# Inside VM, check Rust installation
rustc --version
cargo --version
rustup show

# Reinstall Rust target if needed
rustup target add x86_64-pc-windows-msvc
```

**Permission errors**:
```bash
# On host, verify libvirt group membership
groups | grep libvirt

# If missing, add yourself
sudo usermod -aG libvirt $USER
# Log out and log back in
```

**Network conflicts**:
If your LAN uses 192.168.121.x subnet, edit `Vagrantfile` and uncomment:
```ruby
libvirt.management_network_address = "10.10.10.0/24"
```

## Re-provisioning

Re-run provisioning without destroying VM:

```bash
# All provisioners
vagrant provision

# Specific provisioner
vagrant provision --provision-with configure-rust
```

## VM Management

```bash
vagrant up          # Start VM
vagrant halt        # Stop VM
vagrant reload      # Restart VM (applies Vagrantfile changes)
vagrant destroy     # Delete VM completely
vagrant ssh         # SSH access
vagrant status      # Show VM status
vagrant snapshot save <name>    # Save snapshot
vagrant snapshot restore <name> # Restore snapshot
```

## Development Workflow

**Typical flow**:

1. Make changes in keyrx2 on Linux
2. `vagrant rsync` to sync to VM
3. `vagrant ssh` to connect
4. `cd C:\vagrant_project`
5. `cargo test -p keyrx_daemon --features windows`
6. Fix issues, repeat

**Automated flow** (using script):

```bash
# From project root
./scripts/platform/windows/test_vm.sh
```

This script automatically:
- Checks VM status (starts if needed)
- Syncs files
- Runs Windows tests

## Performance Tips

1. **Use snapshots** - Save after provisioning to avoid re-downloading tools
2. **Keep VM running** - Use `vagrant halt` instead of `destroy` for faster restart
3. **Selective syncing** - Only sync when needed, not automatically
4. **Close GUI** - Use SSH for better performance
5. **Exclude build artifacts** - Already configured in Vagrantfile

## Integration with keyrx2 CI/CD

The VM can be used for local Windows testing before pushing to CI:

```bash
# Test locally before push
./scripts/platform/windows/test_vm.sh

# If tests pass, commit and push
git add .
git commit -m "Fix Windows compatibility"
git push
```

## References

- Main documentation: `/home/rmondo/repos/keyrx2/docs/development/windows-vm-setup.md`
- Project README: `/home/rmondo/repos/keyrx2/README.md`
- Vagrant docs: https://developer.hashicorp.com/vagrant/docs
- vagrant-libvirt: https://vagrant-libvirt.github.io/vagrant-libvirt/
