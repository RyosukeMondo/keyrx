import type { CSSProperties } from 'react';

import type { ViewState } from '../App';

type SidebarProps = {
    activeView: ViewState;
    onSelect: (view: ViewState) => void;
};

const Sidebar = ({ activeView, onSelect }: SidebarProps) => {
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

    const NavItem = ({ label, icon, view }: { label: string; icon: string; view: ViewState }) => (
        <div
            style={{ ...styles.item, ...(activeView === view ? styles.active : {}) }}
            onClick={() => onSelect(view)}
        >
            <span style={styles.icon}>{icon}</span>
            {label}
        </div>
    );

    return (
        <div style={styles.container}>
            <NavItem label="Flow" icon="⚡" view="flow_editor" />
            <NavItem label="Layouts" icon="▦" view="selector" />
            <NavItem label="Devices" icon="⌨" view="editor" /> {/* Placeholder mapping */}
        </div>
    );
};

export default Sidebar;
