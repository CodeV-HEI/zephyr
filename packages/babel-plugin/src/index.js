const fs = require('fs');
const path = require('path');
const { compile } = require('@tailwindcss/oxide');
const { cssToReactNative } = require('./mapping');

module.exports = function () {
    let classesSet = new Set();

    return {
        visitor: {
            JSXAttribute(path) {
                if (path.node.name.name === 'tw' && path.node.value.type === 'StringLiteral') {
                    classesSet.add(path.node.value.value);
                }
            },
        },
        async post() {
            if (classesSet.size === 0) return;

            const outputDir = process.cwd();
            const classArray = Array.from(classesSet);
            const results = {};

            // Résoudre le chemin de la config Tailwind (si elle existe)
            const configPath = fs.existsSync(path.join(outputDir, 'tailwind.config.js'))
                ? path.join(outputDir, 'tailwind.config.js')
                : undefined;

            for (const classString of classArray) {
                // compile retourne du CSS (string) pour la chaîne de classes donnée
                const css = await compile(classString, {
                    config: configPath,
                    // On peut passer d’autres options si besoin (minify, etc.)
                    minify: true,
                });
                // Transformer le CSS en style React Native (base + variantes)
                const rnStyles = cssToReactNative(css);
                results[classString] = rnStyles;
            }

            fs.writeFileSync(
                path.join(outputDir, 'zephyr-precompiled.json'),
                JSON.stringify(results, null, 2)
            );
            console.log(`✅ Zephyr: precompiled ${classArray.length} class strings.`);
        },
    };
};