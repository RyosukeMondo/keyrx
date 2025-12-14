import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';

export type KeymapNodeData = {
    name: string;
    layers: number;
    active: boolean;
};

export function KeymapNode({ data }: NodeProps<Node<KeymapNodeData>>) {
    return (
        <div style={{
            padding: '3px',
            borderRadius: '10px',
            background: data.active ? 'linear-gradient(45deg, #9b59b6, #8e44ad)' : '#333',
            boxShadow: data.active ? '0 0 15px rgba(155, 89, 182, 0.6)' : 'none',
        }}>
            <div style={{
                padding: '12px 18px',
                borderRadius: '8px',
                background: '#191919',
                color: '#fff',
                minWidth: '200px',
                display: 'flex',
                flexDirection: 'column'
            }}>
                <Handle type="target" position={Position.Left} style={{ background: '#fff', width: '10px', height: '10px' }} />

                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
                    <div style={{ fontSize: '10px', textTransform: 'uppercase', color: '#9b59b6', fontWeight: 700 }}>Keymap</div>
                    {data.active && (
                        <div style={{
                            fontSize: '9px',
                            padding: '2px 6px',
                            borderRadius: '8px',
                            background: '#9b59b6',
                            color: '#fff',
                            fontWeight: 600
                        }}>ACTIVE</div>
                    )}
                </div>

                <div style={{ fontWeight: 700, fontSize: '16px', marginBottom: '8px' }}>{data.name}</div>

                <div style={{
                    marginTop: 'auto',
                    display: 'flex',
                    gap: '8px',
                    fontSize: '11px',
                    color: '#888'
                }}>
                    <span>{data.layers} Layers</span>
                    <span>•</span>
                    <span>QWERTY</span>
                </div>
            </div>
        </div>
    );
}
