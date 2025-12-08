export interface LayoutConfig {
    id: string;
    name: string;
    type: 'freeform' | 'grid';
    rows?: number;
    cols?: number;
    keys: Array<{ id: string; label: string; x: number; y: number; w: number; h: number; r?: number; c?: number; }>;
}
