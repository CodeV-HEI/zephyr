import { useCallback } from 'react';
import { useZephyrContext } from './context';

export function useZephyr() {
    const { resolveSync } = useZephyrContext();

    const tw = useCallback(
        (classString: string) => resolveSync(classString),
        [resolveSync]
    );

    return { tw };
}