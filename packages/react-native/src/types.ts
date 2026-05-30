export type CSSDeclaration = {
    property: string;
    value: string;
};

export type Condition = {
    type: 'media' | 'selector' | 'pseudo';
    query: string;
};

export type ResolvedStyles = {
    base: Record<string, string>;
    variants: Array<{
        conditions: Condition[];
        style: Record<string, string>;
    }>;
};