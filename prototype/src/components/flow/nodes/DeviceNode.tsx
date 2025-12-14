import { Handle, Position, type NodeProps } from '@xyflow/react';

export type DeviceNodeData = {
    name: string;
    vid: string;
    pid: string;
    status: 'connected' | 'disconnected';
    wirings?: { id: string; name: string; active: boolean }[];
};

export function DeviceNode({ data }: NodeProps<any>) {
    return (
        <div style={{
            padding: '10px 15px',
            borderRadius: '8px',
            background: data.status === 'connected' ? '#1e1e1e' : '#1a1a1a',
            border: data.status === 'connected' ? '1px solid #333' : '1px dashed #444',
            color: data.status === 'connected' ? '#fff' : '#888',
            minWidth: '220px',
            boxShadow: '0 4px 6px rgba(0,0,0,0.3)',
            opacity: data.status === 'connected' ? 1 : 0.8
        }}>
            {/* Header */}
            <div style={{ display: 'flex', alignItems: 'center', marginBottom: '12px' }}>
                <div style={{
                    width: '32px',
                    height: '32px',
                    background: data.status === 'connected' ? 'rgba(255, 255, 255, 0.1)' : 'rgba(255,255,255,0.05)',
                    borderRadius: '4px',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    marginRight: '10px',
                    fontSize: '18px',
                    filter: data.status === 'connected' ? 'none' : 'grayscale(100%)'
                }}>
                    ⌨️
                </div>
                <div>
                    <div style={{ fontWeight: 600, fontSize: '14px' }}>{data.name}</div>
                    <div style={{ fontSize: '11px', color: '#666' }}>
                        {data.vid}:{data.pid}
                    </div>
                </div>
            </div>

            {/* Wirings / Profiles List */}
            {data.wirings && data.wirings.length > 0 && (
                <div style={{ marginBottom: '8px' }}>
                    <div style={{ fontSize: '9px', textTransform: 'uppercase', color: '#666', marginBottom: '4px', fontWeight: 600 }}>Active Profiles (Priority Order)</div>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                        {data.wirings.map((wiring: { id: string; name: string; active: boolean }, index: number) => (
                            <div key={wiring.id} style={{
                                position: 'relative',
                                fontSize: '11px',
                                padding: '6px 8px',
                                borderRadius: '4px',
                                background: wiring.active ? '#1ab39422' : '#333',
                                border: wiring.active ? '1px solid #1ab39444' : '1px solid #444',
                                color: wiring.active ? '#1ab394' : '#888',
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'space-between'
                            }}>
                                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                                    <span style={{ fontSize: '9px', opacity: 0.5 }}>#{index + 1}</span>
                                    <span>{wiring.name}</span>
                                </div>
                                {/* Source Handle for this specific wiring */}
                                <Handle
                                    type="source"
                                    position={Position.Right}
                                    id={wiring.id}
                                    style={{
                                        background: wiring.active ? '#1ab394' : '#666',
                                        width: '8px',
                                        height: '8px',
                                        right: '-19px',
                                        top: '50%',
                                        transform: 'translateY(-50%)'
                                    }}
                                />
                            </div>
                        ))}
                    </div>
                </div>
            )}

            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '8px' }}>
                <div style={{ fontSize: '10px', textTransform: 'uppercase', opacity: 0.5 }}>
                    Virtual Device
                </div>
                <div style={{
                    fontSize: '9px',
                    padding: '2px 6px',
                    borderRadius: '10px',
                    background: data.status === 'connected' ? 'rgba(46, 204, 113, 0.2)' : 'rgba(127, 140, 141, 0.2)',
                    color: data.status === 'connected' ? '#2ecc71' : '#bdc3c7',
                }}>
                    {data.status === 'connected' ? 'Connected' : 'Offline'}
                </div>
            </div>
        </div>
    );
}
