import { useState, useRef, useEffect } from 'react';
import type { LayoutConfig } from '../types';

interface LayoutEditorProps {
    layout: LayoutConfig;
    onBack: () => void;
    onSave: (layout: LayoutConfig) => void;
}

const LayoutEditor = ({ layout, onBack, onSave }: LayoutEditorProps) => {
    const [keys, setKeys] = useState(layout.keys);
    const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
    const [selectionBox, setSelectionBox] = useState<{ startX: number; startY: number; currentX: number; currentY: number; visible: boolean } | null>(null);
    const [showProperties, setShowProperties] = useState(true);

    // Drag state
    const dragRef = useRef<{
        active: boolean;
        mode: 'move' | 'select' | 'none';
        startX: number;
        startY: number;
        initialKeys: typeof keys;
    }>({ active: false, mode: 'none', startX: 0, startY: 0, initialKeys: [] });

    const canvasRef = useRef<HTMLDivElement>(null);

    // Helper to find next available RC
    const getNextRC = (currentKeys: typeof keys) => {
        const occupied = new Set(currentKeys.map(k => `${k.r || 1}-${k.c || 1}`));
        let r = 1;
        while (r < 100) {
            for (let c = 1; c <= 40; c++) {
                if (!occupied.has(`${r}-${c}`)) return { r, c };
            }
            r++;
        }
        return { r: 1, c: 1 };
    };

    // --- Interaction Handlers ---

    const handleKeyMouseDown = (e: React.MouseEvent, id: string) => {
        e.stopPropagation();

        let newSelection = new Set(selectedIds);
        if (e.shiftKey) {
            if (newSelection.has(id)) {
                newSelection.delete(id);
            } else {
                newSelection.add(id);
            }
        } else {
            if (!newSelection.has(id)) {
                newSelection = new Set([id]);
            }
        }
        setSelectedIds(newSelection);

        dragRef.current = {
            active: true,
            mode: 'move',
            startX: e.clientX,
            startY: e.clientY,
            initialKeys: keys.map(k => ({ ...k }))
        };
    };

    const handleCanvasMouseDown = (e: React.MouseEvent) => {
        if (!canvasRef.current) return;
        const rect = canvasRef.current.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        // Clear selection if not holding Shift
        if (!e.shiftKey) {
            setSelectedIds(new Set());
        }

        dragRef.current = {
            active: true,
            mode: 'select',
            startX: x,
            startY: y,
            initialKeys: []
        };

        setSelectionBox({ startX: x, startY: y, currentX: x, currentY: y, visible: true });
    };

    const handleCanvasDoubleClick = (e: React.MouseEvent) => {
        if (!canvasRef.current) return;
        const rect = canvasRef.current.getBoundingClientRect();

        // Center the key (50x50)
        const x = e.clientX - rect.left - 25;
        const y = e.clientY - rect.top - 25;

        // Snap
        const nx = Math.round(x / 10) * 10;
        const ny = Math.round(y / 10) * 10;

        const { r, c } = getNextRC(keys);

        const newKey = {
            id: `K_${Date.now()}`,
            label: `r${r}c${c}`,
            x: nx,
            y: ny,
            w: 50,
            h: 50,
            r,
            c
        };
        setKeys(prev => [...prev, newKey]);
        setSelectedIds(new Set([newKey.id]));
    };

    const handleKeyDoubleClick = (e: React.MouseEvent, id: string) => {
        e.stopPropagation();
        setKeys(prev => prev.filter(k => k.id !== id));
        setSelectedIds(prev => {
            const next = new Set(prev);
            next.delete(id);
            return next;
        });
    };

    const handleMouseMove = (e: MouseEvent) => {
        if (!dragRef.current.active) return;

        if (dragRef.current.mode === 'move') {
            const dx = e.clientX - dragRef.current.startX;
            const dy = e.clientY - dragRef.current.startY;

            setKeys(prev => {
                return dragRef.current.initialKeys.map(k => {
                    if (selectedIds.has(k.id)) {
                        let nx = k.x + dx;
                        let ny = k.y + dy;
                        // Snap
                        nx = Math.round(nx / 10) * 10;
                        ny = Math.round(ny / 10) * 10;
                        return { ...k, x: nx, y: ny };
                    }
                    return k;
                });
            });
        } else if (dragRef.current.mode === 'select' && canvasRef.current) {
            // Update selection box
            const rect = canvasRef.current.getBoundingClientRect();
            const currentX = e.clientX - rect.left;
            const currentY = e.clientY - rect.top;

            // Update visible box
            setSelectionBox(prev => prev ? { ...prev, currentX, currentY } : null);

            // Calculate intersections
            const startX = Math.min(dragRef.current.startX, currentX);
            const startY = Math.min(dragRef.current.startY, currentY);
            const endX = Math.max(dragRef.current.startX, currentX);
            const endY = Math.max(dragRef.current.startY, currentY);

            const newSelectedIds = new Set(e.shiftKey ? selectedIds : []);

            keys.forEach(k => {
                // Key bounds
                const kx1 = k.x;
                const ky1 = k.y;
                const kx2 = k.x + k.w;
                const ky2 = k.y + k.h;

                // Check intersection
                // !(rect1.right < rect2.left || rect1.left > rect2.right || rect1.bottom < rect2.top || rect1.top > rect2.bottom)
                const intersects = !(kx2 < startX || kx1 > endX || ky2 < startY || ky1 > endY);

                if (intersects) {
                    newSelectedIds.add(k.id);
                }
            });
            setSelectedIds(newSelectedIds);
        }
    };

    const handleMouseUp = () => {
        dragRef.current.active = false;
        dragRef.current.mode = 'none';
        setSelectionBox(null);
    };

    useEffect(() => {
        window.addEventListener('mousemove', handleMouseMove);
        window.addEventListener('mouseup', handleMouseUp);
        return () => {
            window.removeEventListener('mousemove', handleMouseMove);
            window.removeEventListener('mouseup', handleMouseUp);
        };
    }, [selectedIds, keys]);


    // --- Toolbar Actions ---

    const addKey = () => {
        const { r, c } = getNextRC(keys);
        const newKey = {
            id: `K_${Date.now()}`,
            label: `r${r}c${c}`,
            x: 100,
            y: 100,
            w: 50,
            h: 50,
            r,
            c
        };
        setKeys([...keys, newKey]);
        setSelectedIds(new Set([newKey.id]));
    };

    const alignKeys = (alignment: 'left' | 'center' | 'right' | 'top' | 'middle' | 'bottom') => {
        const selected = keys.filter(k => selectedIds.has(k.id));
        if (selected.length < 2) return;

        let targetVal = 0;
        if (alignment === 'left') targetVal = Math.min(...selected.map(k => k.x));
        if (alignment === 'right') targetVal = Math.max(...selected.map(k => k.x + k.w));
        if (alignment === 'center') {
            const minX = Math.min(...selected.map(k => k.x));
            const maxX = Math.max(...selected.map(k => k.x + k.w));
            targetVal = (minX + maxX) / 2;
        }
        if (alignment === 'top') targetVal = Math.min(...selected.map(k => k.y));
        if (alignment === 'bottom') targetVal = Math.max(...selected.map(k => k.y + k.h));
        if (alignment === 'middle') {
            const minY = Math.min(...selected.map(k => k.y));
            const maxY = Math.max(...selected.map(k => k.y + k.h));
            targetVal = (minY + maxY) / 2;
        }

        setKeys(keys.map(k => {
            if (!selectedIds.has(k.id)) return k;
            if (alignment === 'left') return { ...k, x: targetVal };
            if (alignment === 'right') return { ...k, x: targetVal - k.w };
            if (alignment === 'center') return { ...k, x: targetVal - k.w / 2 };
            if (alignment === 'top') return { ...k, y: targetVal };
            if (alignment === 'bottom') return { ...k, y: targetVal - k.h };
            if (alignment === 'middle') return { ...k, y: targetVal - k.h / 2 };
            return k;
        }));
    };

    const distributeKeys = (axis: 'horizontal' | 'vertical') => {
        const selected = keys.filter(k => selectedIds.has(k.id));
        if (selected.length < 3) return;

        if (axis === 'horizontal') {
            const sorted = [...selected].sort((a, b) => a.x - b.x);
            const first = sorted[0];
            const last = sorted[sorted.length - 1];
            // Distribute centers
            const minCenter = first.x + first.w / 2;
            const maxCenter = last.x + last.w / 2;
            const step = (maxCenter - minCenter) / (sorted.length - 1);

            const newMap = new Map();
            sorted.forEach((k, i) => {
                const newCenter = minCenter + i * step;
                newMap.set(k.id, Math.round(newCenter - k.w / 2));
            });

            setKeys(keys.map(k => newMap.has(k.id) ? { ...k, x: newMap.get(k.id) } : k));
        } else {
            const sorted = [...selected].sort((a, b) => a.y - b.y);
            const first = sorted[0];
            const last = sorted[sorted.length - 1];
            const minCenter = first.y + first.h / 2;
            const maxCenter = last.y + last.h / 2;
            const step = (maxCenter - minCenter) / (sorted.length - 1);

            const newMap = new Map();
            sorted.forEach((k, i) => {
                const newCenter = minCenter + i * step;
                newMap.set(k.id, Math.round(newCenter - k.h / 2));
            });

            setKeys(keys.map(k => newMap.has(k.id) ? { ...k, y: newMap.get(k.id) } : k));
        }
    };

    // --- Property Editor Logic ---
    const selectedKey = keys.find(k => selectedIds.has(k.id));
    const multipleSelected = selectedIds.size > 1;

    const deleteSelected = () => {
        setKeys(prev => prev.filter(k => !selectedIds.has(k.id)));
        setSelectedIds(new Set());
    };

    const updateKeyProperty = (prop: string, val: any) => {
        if (!selectedKey) return;

        setKeys(keys.map(k => {
            if (selectedIds.has(k.id)) {
                const newK = { ...k, [prop]: val };
                if (prop === 'r' || prop === 'c') {
                    // Auto label logic
                    const newR = prop === 'r' ? val : (newK.r || 1);
                    const newC = prop === 'c' ? val : (newK.c || 1);

                    // Only update label if it looks like rXcY or Key
                    const currentR = k.r || 1;
                    const currentC = k.c || 1;
                    const defaultLabel = `r${currentR}c${currentC}`;
                    if (k.label === defaultLabel || k.label === 'Key') {
                        newK.label = `r${newR}c${newC}`;
                    }
                }
                return newK;
            }
            return k;
        }));
    };

    const rcOptions = Array.from({ length: 40 }, (_, i) => i + 1);

    return (
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', height: '100%' }}>
            {/* Toolbar */}
            <div style={{
                backgroundColor: 'var(--surface-color)',
                height: '50px',
                borderBottom: '1px solid var(--border-color)',
                display: 'flex',
                alignItems: 'center',
                padding: '0 16px',
                gap: '8px'
            }}>
                <button className="btn btn-text" onClick={onBack} style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>← Back</button>
                <div style={{ width: '1px', height: '20px', backgroundColor: 'var(--border-color)', margin: '0 8px' }}></div>

                <button className="btn btn-text" title="Add Key" onClick={addKey}>+ Key</button>

                <div style={{ width: '1px', height: '20px', backgroundColor: 'var(--border-color)', margin: '0 8px' }}></div>

                {/* Alignment Tools */}
                <button className="btn btn-text" title="Align Left" onClick={() => alignKeys('left')}>⇤</button>
                <button className="btn btn-text" title="Align Center" onClick={() => alignKeys('center')}>⇹</button>
                <button className="btn btn-text" title="Align Right" onClick={() => alignKeys('right')}>⇥</button>
                <button className="btn btn-text" title="Align Top" onClick={() => alignKeys('top')}>⤒</button>
                <button className="btn btn-text" title="Align Middle" onClick={() => alignKeys('middle')}>⤡</button>
                <button className="btn btn-text" title="Align Bottom" onClick={() => alignKeys('bottom')}>⤓</button>

                <div style={{ width: '1px', height: '20px', backgroundColor: 'var(--border-color)', margin: '0 8px' }}></div>

                <button className="btn btn-text" title="Distribute Horizontally" onClick={() => distributeKeys('horizontal')}>H-Dist</button>
                <button className="btn btn-text" title="Distribute Vertically" onClick={() => distributeKeys('vertical')}>V-Dist</button>

                <div style={{ flex: 1 }}></div>
                <button className="btn btn-text" onClick={() => setShowProperties(!showProperties)} style={{ marginRight: '12px', fontSize: '18px' }} title={showProperties ? 'Hide Properties' : 'Show Properties'}>
                    {showProperties ? '»' : '«'}
                </button>
                <button className="btn" onClick={() => onSave({ ...layout, keys })}>Save</button>
            </div>

            <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
                {/* Canvas */}
                <div
                    ref={canvasRef}
                    onMouseDown={handleCanvasMouseDown}
                    onDoubleClick={handleCanvasDoubleClick}
                    style={{
                        flex: 1,
                        backgroundColor: '#202124',
                        position: 'relative',
                        overflow: 'hidden',
                        backgroundImage: 'radial-gradient(#333 1px, transparent 1px)',
                        backgroundSize: '20px 20px',
                    }}
                >
                    {/* Selection Box */}
                    {selectionBox && selectionBox.visible && (
                        <div style={{
                            position: 'absolute',
                            left: Math.min(selectionBox.startX, selectionBox.currentX),
                            top: Math.min(selectionBox.startY, selectionBox.currentY),
                            width: Math.abs(selectionBox.currentX - selectionBox.startX),
                            height: Math.abs(selectionBox.currentY - selectionBox.startY),
                            backgroundColor: 'rgba(137, 180, 248, 0.2)',
                            border: '1px solid var(--primary-color)',
                            pointerEvents: 'none',
                            zIndex: 100
                        }} />
                    )}

                    {keys.map(k => (
                        <div
                            key={k.id}
                            onMouseDown={(e) => handleKeyMouseDown(e, k.id)}
                            onClick={(e) => e.stopPropagation()}
                            onDoubleClick={(e) => handleKeyDoubleClick(e, k.id)}
                            style={{
                                position: 'absolute',
                                left: k.x,
                                top: k.y,
                                width: k.w,
                                height: k.h,
                                backgroundColor: 'var(--surface-color)',
                                border: selectedIds.has(k.id) ? '2px solid var(--primary-color)' : '1px solid var(--border-color)',
                                borderRadius: '6px',
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'center',
                                cursor: 'pointer',
                                userSelect: 'none',
                                color: 'var(--text-color)',
                                fontSize: '12px',
                                zIndex: selectedIds.has(k.id) ? 10 : 1
                            }}
                        >
                            {k.label}
                        </div>
                    ))}
                </div>

                {/* Property Bar */}
                {showProperties && (
                    <div style={{ width: '250px', backgroundColor: 'var(--surface-color)', borderLeft: '1px solid var(--border-color)', padding: '20px' }}>
                        <div className="section-title" style={{ fontSize: '16px' }}>
                            {selectedIds.size === 0 ? 'Properties' : `Properties (${selectedIds.size})`}
                        </div>
                        {selectedKey ? (
                            <>
                                <div style={{ marginBottom: '16px' }}>
                                    <label style={{ display: 'block', fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '6px' }}>Label</label>
                                    <input type="text" value={selectedKey.label} onChange={(e) => updateKeyProperty('label', e.target.value)} disabled={multipleSelected} placeholder={multipleSelected ? 'Multi-select' : ''} />
                                </div>

                                <div style={{ display: 'flex', gap: '10px' }}>
                                    <div style={{ marginBottom: '16px', flex: 1 }}>
                                        <label style={{ display: 'block', fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '6px' }}>Row</label>
                                        <select
                                            value={selectedKey.r || 1}
                                            onChange={(e) => updateKeyProperty('r', parseInt(e.target.value))}
                                            style={{ width: '100%', backgroundColor: 'var(--bg-color)', border: '1px solid var(--border-color)', color: 'var(--text-color)', padding: '8px', borderRadius: '4px' }}
                                        >
                                            {rcOptions.map(o => <option key={o} value={o}>{o}</option>)}
                                        </select>
                                    </div>
                                    <div style={{ marginBottom: '16px', flex: 1 }}>
                                        <label style={{ display: 'block', fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '6px' }}>Column</label>
                                        <select
                                            value={selectedKey.c || 1}
                                            onChange={(e) => updateKeyProperty('c', parseInt(e.target.value))}
                                            style={{ width: '100%', backgroundColor: 'var(--bg-color)', border: '1px solid var(--border-color)', color: 'var(--text-color)', padding: '8px', borderRadius: '4px' }}
                                        >
                                            {rcOptions.map(o => <option key={o} value={o}>{o}</option>)}
                                        </select>
                                    </div>
                                </div>

                                <div style={{ marginBottom: '16px' }}>
                                    <label style={{ display: 'block', fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '6px' }}>Position X</label>
                                    <input type="number" value={selectedKey.x} onChange={(e) => updateKeyProperty('x', parseInt(e.target.value))} />
                                </div>
                                <div style={{ marginBottom: '16px' }}>
                                    <label style={{ display: 'block', fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '6px' }}>Position Y</label>
                                    <input type="number" value={selectedKey.y} onChange={(e) => updateKeyProperty('y', parseInt(e.target.value))} />
                                </div>
                                <div style={{ marginBottom: '16px' }}>
                                    <label style={{ display: 'block', fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '6px' }}>Width</label>
                                    <input type="number" value={selectedKey.w} onChange={(e) => updateKeyProperty('w', parseInt(e.target.value))} />
                                </div>
                                <div style={{ marginBottom: '16px' }}>
                                    <label style={{ display: 'block', fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '6px' }}>Height</label>
                                    <input type="number" value={selectedKey.h} onChange={(e) => updateKeyProperty('h', parseInt(e.target.value))} />
                                </div>

                                <button
                                    className="btn"
                                    style={{ marginTop: '20px', backgroundColor: '#d93025', color: 'white', width: '100%' }}
                                    onClick={deleteSelected}
                                >
                                    Delete {multipleSelected ? `(${selectedIds.size})` : ''}
                                </button>
                            </>
                        ) : (
                            <div style={{ color: 'var(--text-secondary)', fontSize: '14px' }}>Select a key to edit properties</div>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
};

export default LayoutEditor;
