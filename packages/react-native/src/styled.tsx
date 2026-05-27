import React, { forwardRef } from 'react';
import { View, Text, TouchableOpacity } from 'react-native';
import { useZephyr } from './useZephyr';

type StyledComponent<T, P> = React.ForwardRefExoticComponent<
    P & { tw?: string; style?: any } & React.RefAttributes<T>
>;

function createStyled<C extends React.ComponentType<any>>(
    Comp: C
): StyledComponent<React.ElementRef<C>, React.ComponentPropsWithoutRef<C>> {
    return forwardRef((props: any, ref) => {
        const { tw: className, style, ...rest } = props;
        const { tw } = useZephyr();
        const generated = className ? tw(className) : {};
        return <Comp ref={ref} style={[generated, style]} {...rest} />;
    });
}

export const StyledView = createStyled(View);
export const StyledText = createStyled(Text);
export const StyledTouchableOpacity = createStyled(TouchableOpacity);