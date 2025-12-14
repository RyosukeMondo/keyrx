import { useCallback } from 'react';
import {
    ReactFlow,
    Controls,
    Background,
    useNodesState,
    useEdgesState,
    addEdge,
    type Connection,

    BackgroundVariant,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

import { DeviceNode } from './nodes/DeviceNode';
import { WiringNode } from './nodes/WiringNode';
import { LayoutNode } from './nodes/LayoutNode';
import { KeymapNode } from './nodes/KeymapNode';

const nodeTypes = {
    device: DeviceNode,
    wiring: WiringNode,
    layout: LayoutNode,
    keymap: KeymapNode,
} as any;

const initialNodes = [
    {
        id: 'device-1',
        type: 'device',
        position: { x: 50, y: 100 },
        data: {
            name: 'Keychron Q1',
            vid: '0x3434',
            pid: '0x0120',
            status: 'connected',
            wirings: [
                { id: 'w1', name: 'Main Config', active: true },
                { id: 'w2', name: 'Fallback Config', active: false }
            ]
        },
    },
    // Chain 1 (Main)
    {
        id: 'layout-1',
        type: 'layout',
        position: { x: 450, y: 50 },
        data: { name: 'ANSI 104', type: 'preset' },
    },
    {
        id: 'keymap-1',
        type: 'keymap',
        position: { x: 750, y: 50 },
        data: { name: 'Gaming Layer', layers: 4, active: true },
    },
    // Chain 2 (Fallback)
    {
        id: 'layout-2',
        type: 'layout',
        position: { x: 450, y: 300 },
        data: { name: 'ISO 105', type: 'preset' },
    },
    {
        id: 'keymap-2',
        type: 'keymap',
        position: { x: 750, y: 300 },
        data: { name: 'Office Layer', layers: 2, active: false },
    },
];

const initialEdges = [
    // Priority 1 Flow
    { id: 'e1-l1', source: 'device-1', sourceHandle: 'w1', target: 'layout-1', style: { stroke: '#1ab394', strokeWidth: 2 }, animated: true },
    { id: 'el1-k1', source: 'layout-1', target: 'keymap-1', style: { stroke: '#9b59b6', strokeWidth: 2 }, animated: true },

    // Priority 2 Flow (Dimmed)
    { id: 'e1-l2', source: 'device-1', sourceHandle: 'w2', target: 'layout-2', style: { stroke: '#555', strokeWidth: 2, strokeDasharray: '5,5' } },
    { id: 'el2-k2', source: 'layout-2', target: 'keymap-2', style: { stroke: '#555', strokeWidth: 2 } },
];

export default function FlowEditor() {
    const [nodes, , onNodesChange] = useNodesState(initialNodes);
    const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

    const onConnect = useCallback(
        (params: Connection) => setEdges((eds) => addEdge({ ...params }, eds)),
        [setEdges],
    );

    return (
        <div style={{ width: '100%', height: '100%', background: '#121212' }}>
            <ReactFlow
                nodes={nodes}
                edges={edges}
                onNodesChange={onNodesChange}
                onEdgesChange={onEdgesChange}
                onConnect={onConnect}
                nodeTypes={nodeTypes}
                fitView
                colorMode="dark"
            >
                <Background variant={BackgroundVariant.Dots} gap={12} size={1} color="#333" />
                <Controls style={{ fill: '#fff' }} />
            </ReactFlow>
        </div>
    );
}
