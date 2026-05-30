const { parse } = require('css-tree');

function cssToReactNative(cssString) {
    const ast = parse(cssString, { positions: false });
    const base = {};
    const variants = [];

    function extractDeclarations(blockNode) {
        const style = {};
        if (!blockNode.children) return style;
        for (const decl of blockNode.children) {
            if (decl.type === 'Declaration') {
                const prop = decl.property.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
                // handle multiple values (e.g., "1px solid black") – simplified
                let value = '';
                if (decl.value.children) {
                    const parts = [];
                    for (const node of decl.value.children) {
                        if (node.type === 'Raw') parts.push(node.value);
                        else if (node.type === 'Identifier') parts.push(node.name);
                        else if (node.type === 'Number') parts.push(node.value);
                        else if (node.type === 'Dimension') parts.push(`${node.value}${node.unit}`);
                        else if (node.type === 'Percentage') parts.push(`${node.value}%`);
                        else if (node.type === 'Function') parts.push(`${node.name}(${node.children?.toArray().map(n => n.name).join(',')})`);
                        else if (node.value) parts.push(node.value);
                    }
                    value = parts.join(' ');
                }
                style[prop] = value;
            }
        }
        return style;
    }

    function walk(node) {
        if (node.type === 'Rule') {
            // Extract selector (first class name from the first selector)
            let className = null;
            if (node.prelude && node.prelude.children) {
                const firstSelector = node.prelude.children.first;
                if (firstSelector && firstSelector.children) {
                    const firstClass = firstSelector.children.find(c => c.type === 'ClassSelector');
                    if (firstClass) className = firstClass.name;
                }
            }
            const style = extractDeclarations(node.block);
            if (className === null || className === '\\:root') {
                Object.assign(base, style);
            } else if (className.startsWith(':')) { // pseudo-class like :hover, :focus
                variants.push({
                    conditions: [{ type: 'pseudo', query: className.slice(1) }],
                    style
                });
            } else if (className.includes('\\:')) { // escaped colon, e.g., dark\:bg-red
                const raw = className.replace(/\\:/g, ':');
                variants.push({
                    conditions: [{ type: 'selector', query: raw }],
                    style
                });
            }
        } else if (node.type === 'Atrule' && node.name === 'media') {
            let condition = '';
            if (node.prelude && node.prelude.children) {
                const parts = [];
                for (const child of node.prelude.children) {
                    if (child.type === 'Raw') parts.push(child.value);
                    else if (child.type === 'Identifier') parts.push(child.name);
                    else if (child.type === 'Dimension') parts.push(`${child.value}${child.unit}`);
                    else if (child.type === 'Operator') parts.push(child.value);
                }
                condition = parts.join(' ');
            }
            if (node.block && node.block.children) {
                for (const child of node.block.children) {
                    if (child.type === 'Rule') {
                        const style = extractDeclarations(child.block);
                        variants.push({
                            conditions: [{ type: 'media', query: condition }],
                            style
                        });
                    }
                }
            }
        }
        if (node.children) node.children.forEach(walk);
    }

    walk(ast);
    return { base, variants };
}