import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';

export type WiringNodeData = {
    name: string;
    mcu: string;
    matrix: string;
};

export function WiringNode({ data }: NodeProps<Node<WiringNodeData>>) {
    return (
        <div style={{
            padding: '10px 15px',
            borderRadius: '8px',
            background: '#1ab394', // Teal accent
            border: '1px solid #158f76',
            color: '#fff',
            minWidth: '160px',
            boxShadow: '0 4px 6px rgba(0,0,0,0.2)'
        }}>
            <Handle type="target" position={Position.Left} style={{ background: '#fff' }} />

            <div style={{ fontSize: '10px', textTransform: 'uppercase', opacity: 0.8, fontWeight: 700 }}>Wiring</div>
            <div style={{ fontWeight: 600, fontSize: '14px', marginBottom: '6px' }}>{data.name}</div>

            <div style={{ fontSize: '11px', opacity: 0.9 }}>
                <div>MCU: {data.mcu}</div>
                <div>Matrix: {data.matrix}</div>
            </div>

            <Handle type="source" position={Position.Right} style={{ background: '#fff' }} />
        </div>
    );
}
