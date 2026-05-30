import React from 'react';
import { SafeAreaView, Text } from 'react-native';
import { ZephyrProvider, StyledView, StyledText } from '@zephyr/react-native';
import precompiled from './zephyr-precompiled.json';

export default function App() {
  return (
    <ZephyrProvider precompiledStyles={precompiled} baseFontSize={16}>
      <SafeAreaView>
        <StyledView tw="p-4 bg-blue-500 rounded-lg">
          <StyledText tw="text-white text-lg font-bold">
            Hello Zephyr + Tailwind v4!
          </StyledText>
        </StyledView>
      </SafeAreaView>
    </ZephyrProvider>
  );
}