import { useState } from 'react';

interface GridGeneratorProps {
    onSelect: (rows: number, cols: number) => void;
}

const GridGenerator = ({ onSelect }: GridGeneratorProps) => {
    const MAX_ROWS = 6;
    const MAX_COLS = 20;

    const [hover, setHover] = useState({ r: 0, c: 0 });

    const handleMouseEnter = (r: number, c: number) => {
        setHover({ r, c });
    };

    const handleContainerLeave = () => {
        setHover({ r: 0, c: 0 });
    };

    return (
        <div
            style={{
                flex: 2,
                display: 'flex',
                flexDirection: 'column',
                backgroundColor: 'var(--surface-color)',
                borderRadius: '16px',
                padding: '24px',
                cursor: 'default'
            }}
        >
            <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '16px' }}>
                <span style={{ fontSize: '24px' }}>▦</span>
                <div>
                    <div style={{ fontSize: '18px', fontWeight: 600 }}>Grid Generator</div>
                    <div style={{ color: 'var(--text-secondary)', fontSize: '14px' }}>Hover to pick size</div>
                </div>
            </div>

            <div
                style={{
                    flex: 1,
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    justifyContent: 'center'
                }}
                onMouseLeave={handleContainerLeave}
            >
                <div style={{ marginBottom: '10px', fontSize: '14px', color: 'var(--text-secondary)' }}>
                    {hover.r > 0 ? `${hover.r} x ${hover.c}` : 'Select Size'}
                </div>
                <div style={{
                    display: 'grid',
                    gap: '2px',
                    gridTemplateColumns: `repeat(${MAX_COLS}, 1fr)`
                }}>
                    {Array.from({ length: MAX_ROWS * MAX_COLS }).map((_, i) => {
                        const r = Math.floor(i / MAX_COLS) + 1;
                        const c = (i % MAX_COLS) + 1;
                        const isActive = r <= hover.r && c <= hover.c;

                        return (
                            <div
                                key={i}
                                style={{
                                    width: '18px',
                                    height: '18px',
                                    backgroundColor: isActive ? 'var(--primary-color)' : 'rgba(255,255,255,0.1)',
                                    border: isActive ? '1px solid var(--primary-color)' : '1px solid rgba(255,255,255,0.2)',
                                    borderRadius: '2px',
                                    cursor: 'pointer',
                                    transition: 'all 0.05s'
                                }}
                                onMouseEnter={() => handleMouseEnter(r, c)}
                                onClick={() => onSelect(r, c)}
                            />
                        );
                    })}
                </div>
            </div>
        </div>
    );
};

export default GridGenerator;
