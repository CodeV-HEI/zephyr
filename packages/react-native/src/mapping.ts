export function declarationsToRN(
    declarations: Record<string, string>,
    baseFontSize = 16
): Record<string, unknown> {
    const result: Record<string, unknown> = {};
    for (const [prop, value] of Object.entries(declarations)) {
        const rnProp = prop.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
        if (rnProp === 'float') continue;
        result[rnProp] = convertValue(value, baseFontSize);
    }
    return result;
}

function convertValue(value: string, baseFontSize: number): string | number {
    if (/^-?\d+%$/.test(value)) return value;
    const rem = value.match(/^(-?[\d.]+)rem$/);
    if (rem) return parseFloat(rem[1]) * baseFontSize;
    const px = value.match(/^(-?[\d.]+)px$/);
    if (px) return parseFloat(px[1]);
    return value;
}