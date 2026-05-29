const { compileClasses } = require('@zephyr/core');
const fs = require('fs');
const path = require('path');

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
            const outputDir = process.cwd();
            const classArray = Array.from(classesSet);
            if (classArray.length === 0) return;

            const results = await Promise.all(classArray.map(cls => compileClasses(cls)));
            const map = {};
            classArray.forEach((cls, idx) => {
                map[cls] = results[idx];
            });
            fs.writeFileSync(
                path.join(outputDir, 'zephyr-precompiled.json'),
                JSON.stringify(map, null, 2)
            );
            console.log(`✅ Zephyr: precompiled ${classArray.length} class strings.`);
        },
    };
};