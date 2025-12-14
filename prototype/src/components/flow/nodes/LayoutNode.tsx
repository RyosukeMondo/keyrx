import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';

export type LayoutNodeData = {
    name: string;
    type: string;
};

export function LayoutNode({ data }: NodeProps<Node<LayoutNodeData>>) {
    return (
        <div style={{
            padding: '0',
            borderRadius: '8px',
            background: '#2c3e50',
            border: '1px solid #34495e',
            color: '#fff',
            minWidth: '180px',
            overflow: 'hidden',
            boxShadow: '0 4px 6px rgba(0,0,0,0.3)'
        }}>
            <Handle type="target" position={Position.Left} style={{ background: '#fff' }} />

            <div style={{ padding: '10px 15px', background: '#34495e', borderBottom: '1px solid #2c3e50' }}>
                <div style={{ fontSize: '10px', textTransform: 'uppercase', opacity: 0.7, fontWeight: 700 }}>Layout</div>
                <div style={{ fontWeight: 600, fontSize: '14px' }}>{data.name}</div>
            </div>

            <div style={{ padding: '15px', display: 'flex', justifyContent: 'center' }}>
                {/* Simple visual representation of a keyboard layout */}
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(5, 1fr)', gap: '4px' }}>
                    {[...Array(15)].map((_, i) => (
                        <div key={i} style={{
                            width: '12px',
                            height: '12px',
                            background: 'rgba(255,255,255,0.1)',
                            borderRadius: '2px'
                        }} />
                    ))}
                </div>
            </div>

            <Handle type="source" position={Position.Right} style={{ background: '#fff' }} />
        </div>
    );
}
