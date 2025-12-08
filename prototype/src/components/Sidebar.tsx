import { CSSProperties } from 'react';

const Sidebar = () => {
    const styles: { [key: string]: CSSProperties } = {
        container: {
            width: '80px',
            backgroundColor: '#111315',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            paddingTop: '20px',
            borderRight: '1px solid var(--border-color)',
        },
        item: {
            width: '50px',
            height: '50px',
            borderRadius: '12px',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            marginBottom: '16px',
            cursor: 'pointer',
            color: 'var(--text-secondary)',
            fontSize: '10px',
        },
        active: {
            backgroundColor: 'var(--primary-container)',
            color: 'var(--primary-color)',
        },
        icon: {
            fontSize: '24px',
            marginBottom: '4px',
            display: 'block',
        }
    };

    const NavItem = ({ label, icon, active = false }: { label: string; icon: string; active?: boolean }) => (
        <div style={{ ...styles.item, ...(active ? styles.active : {}) }}>
            <span style={styles.icon}>{icon}</span>
            {label}
        </div>
    );

    return (
        <div style={styles.container}>
            <NavItem label="Devices" icon="⌨" />
            <NavItem label="Layouts" icon="▦" active />
            <NavItem label="Wiring" icon="∿" />
            <NavItem label="Map" icon="⊞" />
        </div>
    );
};

export default Sidebar;
