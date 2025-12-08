import GridGenerator from './GridGenerator';
import type { LayoutConfig } from '../types';

interface LayoutSelectorProps {
    layouts: LayoutConfig[];
    onSelect: (config: LayoutConfig) => void;
    onDelete: (id: string) => void;
}

const LayoutSelector = ({ layouts, onSelect, onDelete }: LayoutSelectorProps) => {

    const createFreeform = () => {
        onSelect({
            id: `L_${Date.now()}`,
            name: 'New Custom Layout',
            type: 'freeform',
            keys: [{ id: 'K_1', label: 'Key', x: 100, y: 100, w: 50, h: 50 }]
        });
    };

    const createGrid = (rows: number, cols: number) => {
        const keys = [];
        const unit = 60;
        const startX = 100;
        const startY = 100;

        for (let r = 0; r < rows; r++) {
            for (let c = 0; c < cols; c++) {
                keys.push({
                    id: `K_${r}_${c}`,
                    label: `r${r + 1}c${c + 1}`,
                    x: startX + c * unit,
                    y: startY + r * unit,
                    w: 50,
                    h: 50,
                    r: r + 1,
                    c: c + 1
                });
            }
        }

        onSelect({
            id: `L_${Date.now()}`,
            name: `New Grid ${rows}x${cols}`,
            type: 'grid',
            rows,
            cols,
            keys
        });
    };

    return (
        <div className="view">
            <div className="section-title">Create New Layout</div>

            <div style={{ display: 'flex', gap: '20px', marginBottom: '40px', height: '250px' }}>
                {/* Freeform Card */}
                <div
                    onClick={createFreeform}
                    style={{
                        flex: 1,
                        background: 'linear-gradient(135deg, rgba(66, 133, 244, 0.15), var(--surface-color) 60%)',
                        borderRadius: '16px',
                        padding: '24px',
                        display: 'flex',
                        flexDirection: 'column',
                        justifyContent: 'flex-end',
                        cursor: 'pointer',
                        position: 'relative'
                    }}
                >
                    <div style={{ position: 'absolute', top: 20, left: 20, fontSize: '40px', color: 'var(--primary-color)', opacity: 0.8 }}>✎</div>
                    <div style={{ fontSize: '18px', fontWeight: 600, marginBottom: '8px' }}>Freeform Layout</div>
                    <div style={{ color: 'var(--text-secondary)', fontSize: '14px' }}>Start from scratch</div>
                </div>

                {/* Grid Generator */}
                <GridGenerator onSelect={createGrid} />
            </div>

            <div className="section-title">Saved Layouts</div>
            <div style={{ display: 'flex', gap: '16px', flexWrap: 'wrap' }}>
                {layouts.map(layout => (
                    <div key={layout.id}
                        onClick={() => onSelect(layout)}
                        style={{
                            backgroundColor: 'var(--surface-color)',
                            borderRadius: '16px',
                            padding: '24px',
                            height: '120px',
                            width: '200px',
                            cursor: 'pointer',
                            position: 'relative'
                        }}>
                        <div
                            onClick={(e) => { e.stopPropagation(); onDelete(layout.id); }}
                            style={{
                                position: 'absolute',
                                top: '10px',
                                right: '10px',
                                width: '24px',
                                height: '24px',
                                borderRadius: '50%',
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'center',
                                color: 'var(--text-secondary)',
                                fontSize: '16px',
                                opacity: 0.6
                            }}
                            title="Delete Layout"
                            onMouseEnter={e => e.currentTarget.style.opacity = '1'}
                            onMouseLeave={e => e.currentTarget.style.opacity = '0.6'}
                        >
                            ×
                        </div>
                        <div style={{
                            border: '2px solid var(--text-secondary)',
                            borderRadius: '4px',
                            width: '24px',
                            height: '24px',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            marginBottom: '10px'
                        }}>{layout.name[0]}</div>
                        <div style={{ fontSize: '16px', fontWeight: 600, marginBottom: '8px', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{layout.name}</div>
                        <div style={{ color: 'var(--text-secondary)', fontSize: '14px' }}>{layout.keys.length} keys</div>
                    </div>
                ))}
            </div>
        </div>
    );
};

export default LayoutSelector;
