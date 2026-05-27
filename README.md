# 🌬️ Zephyr

![Zephyr Logo](./public/ico/zephyr_logo.png)  
*Tailwind CSS v4 natively in React Native – powered by Rust + WASM*

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](http://makeapullrequest.com)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.4%2B-blue)](https://www.typescriptlang.org/)
[![React Native](https://img.shields.io/badge/React%20Native-0.85%2B-61DAFB)](https://reactnative.dev/)

**Zephyr** est un framework qui embarque le compilateur **Tailwind CSS v4** (écrit en Rust) directement dans vos applications React Native, via **WebAssembly**. Il permet d’utiliser **toutes les classes Tailwind v4** (utilitaires, variantes, thèmes, dark mode, breakpoints) sans dépendre de PostCSS, et avec des performances optimales grâce à la pré‑compilation statique.

---

## ✨ Caractéristiques

- ⚡ **Cœur ultra‑rapide** : compilation en Rust, exposée en WASM.
- 📱 **API React Native native** : composants `StyledView`, `StyledText`, `StyledTouchableOpacity` avec la prop `tw`.
- 🔧 **Pré‑compilation statique** (via plugin Babel) : zéro surcharge runtime.
- 🎨 **Support complet de Tailwind v4** : toutes les classes, variantes (`hover:`, `dark:`, `sm:`, …), valeurs arbitraires.
- 🧩 **Monorepo prêt à l’emploi** : packages `@zephyr/core`, `@zephyr/react-native`, `@zephyr/babel-plugin`.
- 🧪 **Exemple d’intégration** : `examples/rnDemo`.

---

## 📦 Installation

### 1. Prérequis

- Node.js 20+ et pnpm
- Rust (1.85+) et `wasm-pack`
- Tailwind CSS CLI installé globalement (`npm install -g tailwindcss`)

### 2. Cloner le monorepo

```bash
git clone https://github.com/CodeV-HEI/zephyr.git
cd zephyr
pnpm install
```

### 3. Compiler le cœur Rust/WASM

```bash
cd packages/core
wasm-pack build --target bundler --out-dir pkg --no-opt
pnpm run build:ts
cd ../..
```

### 4. Lier les packages (en développement)

```bash
pnpm install
```

---

## 🚀 Utilisation dans une application React Native

### Créer un projet RN

```bash
npx @react-native-community/cli init MyApp --skip-install
cd MyApp
pnpm add @zephyr/react-native @zephyr/babel-plugin
```

### Configurer Babel et Metro

**`babel.config.js`** :
```js
module.exports = {
  presets: ['module:metro-react-native-babel-preset'],
  plugins: ['@zephyr/babel-plugin'],
};
```

**`metro.config.js`** :
```js
const {getDefaultConfig} = require('@react-native/metro-config');

module.exports = (() => {
  const config = getDefaultConfig(__dirname);
  config.transformer.babelTransformerPath = require.resolve('@zephyr/babel-plugin');
  return config;
})();
```

### Utiliser dans votre `App.tsx`

```tsx
import React from 'react';
import {SafeAreaView, ScrollView} from 'react-native';
import {ZephyrProvider, StyledView, StyledText, StyledTouchableOpacity} from '@zephyr/react-native';
import precompiled from './zephyr-precompiled.json';

export default function App() {
  return (
    <ZephyrProvider precompiledStyles={new Map(Object.entries(precompiled))}>
      <SafeAreaView style={{flex: 1}}>
        <ScrollView contentContainerStyle={{padding: 16}}>
          <StyledView tw="flex-1 bg-white dark:bg-gray-900 p-4 rounded-2xl shadow-lg">
            <StyledText tw="text-2xl font-bold text-blue-600 mb-2">
              Bienvenue sur Zéphyr
            </StyledText>
            <StyledText tw="text-base text-gray-700 dark:text-gray-300">
              Toutes les classes Tailwind v4, directement dans React Native,
              compilées par un cœur Rust ⚡
            </StyledText>
            <StyledTouchableOpacity
              tw="mt-6 bg-blue-500 hover:bg-blue-600 active:bg-blue-700 px-6 py-3 rounded-full self-start"
              onPress={() => alert('Pressé !')}>
              <StyledText tw="text-white font-semibold">Appuyez ici</StyledText>
            </StyledTouchableOpacity>
          </StyledView>
        </ScrollView>
      </SafeAreaView>
    </ZephyrProvider>
  );
}
```

Le plugin Babel générera automatiquement `zephyr-precompiled.json` à la racine de votre projet lors du premier build.

---

## 🧰 Linting et qualité de code

Ajoutez ces fichiers à la racine du monorepo pour garantir un code propre.

### Installation des outils

```bash
pnpm add -D -w eslint prettier @typescript-eslint/eslint-plugin @typescript-eslint/parser eslint-config-prettier eslint-plugin-react eslint-plugin-react-native
```

### `.eslintrc.js`

```js
module.exports = {
  root: true,
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:react/recommended',
    'plugin:react-native/all',
    'prettier',
  ],
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 'latest',
    sourceType: 'module',
    ecmaFeatures: {
      jsx: true,
    },
  },
  env: {
    'react-native/react-native': true,
    es2022: true,
    node: true,
  },
  settings: {
    react: {
      version: 'detect',
    },
  },
  rules: {
    'react/react-in-jsx-scope': 'off',
    '@typescript-eslint/no-explicit-any': 'warn',
  },
};
```

### `.prettierrc.js`

```js
module.exports = {
  arrowParens: 'avoid',
  bracketSameLine: true,
  bracketSpacing: false,
  singleQuote: true,
  trailingComma: 'all',
  semi: true,
};
```

### Mettre à jour `package.json` racine

```json
"scripts": {
  "lint": "eslint . --ext .js,.jsx,.ts,.tsx",
  "format": "prettier --write ."
}
```

La GitHub Action `lint.yml` (déjà présente) utilisera ces configurations.

---

## 🤝 Contribution

Les contributions sont les bienvenues ! Merci de lire le fichier [CONTRIBUTING.md](CONTRIBUTING.md) (à créer) et de suivre le code de conduite.

---

## 📄 Licence

MIT © [Zephyr Contributors](LICENSE)

---

## 🙏 Remerciements

- L’équipe de **Tailwind CSS** pour leur compilateur v4.  
- La communauté **Rust** et **wasm-bindgen**.  
- **React Native** pour son écosystème.

---

**Zephyr** – *Native Tailwind, Rust-powered.*