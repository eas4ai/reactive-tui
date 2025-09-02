/**
 * React-style Hooks for Reactive TUI
 * 
 * Provides state management and lifecycle hooks for functional components.
 */

import { Component } from './component';

/**
 * Hook context for managing component state and effects
 */
class HookContext {
    private static currentComponent: Component | null = null;
    private static hookIndex: number = 0;
    private static hooks: Map<Component, any[]> = new Map();
    
    static setCurrentComponent(component: Component) {
        this.currentComponent = component;
        this.hookIndex = 0;
    }
    
    static getHook<T>(initialValue?: T): [T, number] {
        if (!this.currentComponent) {
            throw new Error('Hooks can only be called within a component');
        }
        
        const component = this.currentComponent;
        const index = this.hookIndex++;
        
        if (!this.hooks.has(component)) {
            this.hooks.set(component, []);
        }
        
        const componentHooks = this.hooks.get(component)!;
        
        if (componentHooks.length <= index) {
            componentHooks.push(initialValue);
        }
        
        return [componentHooks[index], index];
    }
    
    static setHook(component: Component, index: number, value: any) {
        const componentHooks = this.hooks.get(component);
        if (componentHooks) {
            componentHooks[index] = value;
        }
    }
    
    static cleanup(component: Component) {
        this.hooks.delete(component);
    }
}

/**
 * State hook for managing component state
 * 
 * @example
 * ```typescript
 * const [count, setCount] = useState(0);
 * setCount(count + 1);
 * ```
 */
export function useState<T>(initialValue: T): [T, (value: T | ((prev: T) => T)) => void] {
    const [value, index] = HookContext.getHook(initialValue);
    const component = HookContext['currentComponent']!;
    
    const setValue = (newValue: T | ((prev: T) => T)) => {
        const actualValue = typeof newValue === 'function' 
            ? (newValue as (prev: T) => T)(value)
            : newValue;
        
        HookContext.setHook(component, index, actualValue);
        component.forceUpdate();
    };
    
    return [value, setValue];
}

/**
 * Effect hook for side effects
 * 
 * @example
 * ```typescript
 * useEffect(() => {
 *     console.log('Component mounted');
 *     return () => console.log('Component unmounted');
 * }, []);
 * ```
 */
export function useEffect(effect: () => void | (() => void), deps?: any[]): void {
    const [prevDeps, index] = HookContext.getHook<{
        deps?: any[];
        cleanup?: () => void;
    }>({ deps });
    
    const component = HookContext['currentComponent']!;
    
    // Check if dependencies changed
    const depsChanged = !prevDeps.deps || !deps || 
        deps.length !== prevDeps.deps.length ||
        deps.some((dep, i) => dep !== prevDeps.deps![i]);
    
    if (depsChanged) {
        // Cleanup previous effect
        if (prevDeps.cleanup) {
            prevDeps.cleanup();
        }
        
        // Run new effect
        const cleanup = effect();
        
        HookContext.setHook(component, index, {
            deps,
            cleanup: cleanup || undefined
        });
    }
}

/**
 * Memoization hook for expensive computations
 * 
 * @example
 * ```typescript
 * const expensiveValue = useMemo(() => {
 *     return computeExpensiveValue(a, b);
 * }, [a, b]);
 * ```
 */
export function useMemo<T>(factory: () => T, deps: any[]): T {
    const [memoized, index] = HookContext.getHook<{
        value?: T;
        deps?: any[];
    }>({});
    
    const component = HookContext['currentComponent']!;
    
    // Check if dependencies changed
    const depsChanged = !memoized.deps ||
        deps.length !== memoized.deps.length ||
        deps.some((dep, i) => dep !== memoized.deps![i]);
    
    if (depsChanged || memoized.value === undefined) {
        const value = factory();
        HookContext.setHook(component, index, { value, deps });
        return value;
    }
    
    return memoized.value;
}

/**
 * Callback hook for memoizing functions
 * 
 * @example
 * ```typescript
 * const handleClick = useCallback(() => {
 *     console.log(count);
 * }, [count]);
 * ```
 */
export function useCallback<T extends (...args: any[]) => any>(
    callback: T,
    deps: any[]
): T {
    return useMemo(() => callback, deps);
}

/**
 * Ref hook for accessing DOM elements or storing mutable values
 * 
 * @example
 * ```typescript
 * const inputRef = useRef<HTMLInputElement>(null);
 * ```
 */
export function useRef<T>(initialValue: T): { current: T } {
    const [ref] = HookContext.getHook({ current: initialValue });
    return ref;
}

/**
 * Reducer hook for complex state management
 * 
 * @example
 * ```typescript
 * const [state, dispatch] = useReducer(reducer, initialState);
 * dispatch({ type: 'increment' });
 * ```
 */
export function useReducer<S, A>(
    reducer: (state: S, action: A) => S,
    initialState: S
): [S, (action: A) => void] {
    const [state, setState] = useState(initialState);
    
    const dispatch = useCallback((action: A) => {
        setState(prevState => reducer(prevState, action));
    }, [reducer]);
    
    return [state, dispatch];
}

/**
 * Context for sharing values between components
 */
export class Context<T> {
    private value: T;
    private subscribers: Set<Component> = new Set();
    
    constructor(defaultValue: T) {
        this.value = defaultValue;
    }
    
    getValue(): T {
        return this.value;
    }
    
    setValue(value: T) {
        this.value = value;
        this.notify();
    }
    
    subscribe(component: Component) {
        this.subscribers.add(component);
    }
    
    unsubscribe(component: Component) {
        this.subscribers.delete(component);
    }
    
    private notify() {
        this.subscribers.forEach(component => component.forceUpdate());
    }
}

/**
 * Create a context
 * 
 * @example
 * ```typescript
 * const ThemeContext = createContext('light');
 * ```
 */
export function createContext<T>(defaultValue: T): Context<T> {
    return new Context(defaultValue);
}

/**
 * Use context hook
 * 
 * @example
 * ```typescript
 * const theme = useContext(ThemeContext);
 * ```
 */
export function useContext<T>(context: Context<T>): T {
    const component = HookContext['currentComponent']!;
    
    useEffect(() => {
        context.subscribe(component);
        return () => context.unsubscribe(component);
    }, [context]);
    
    return context.getValue();
}

/**
 * Layout effect hook (runs synchronously after DOM mutations)
 */
export function useLayoutEffect(effect: () => void | (() => void), deps?: any[]): void {
    // In terminal context, this behaves the same as useEffect
    // but conceptually runs before the screen is painted
    useEffect(effect, deps);
}

/**
 * Imperative handle hook for exposing methods to parent components
 */
export function useImperativeHandle<T>(
    ref: { current: T | null },
    createHandle: () => T,
    deps?: any[]
): void {
    useEffect(() => {
        ref.current = createHandle();
    }, deps);
}

/**
 * Debug value hook for React DevTools
 */
export function useDebugValue<T>(value: T, format?: (value: T) => any): void {
    // In production, this is a no-op
    // Could be used for logging in development
    if (process.env.NODE_ENV === 'development') {
        console.debug('Debug value:', format ? format(value) : value);
    }
}

/**
 * ID hook for generating unique IDs
 */
let idCounter = 0;
export function useId(): string {
    const [id] = useState(() => `rtui-${++idCounter}`);
    return id;
}

/**
 * Deferred value hook for non-urgent updates
 */
export function useDeferredValue<T>(value: T): T {
    const [deferredValue, setDeferredValue] = useState(value);
    
    useEffect(() => {
        const timeout = setTimeout(() => {
            setDeferredValue(value);
        }, 0);
        
        return () => clearTimeout(timeout);
    }, [value]);
    
    return deferredValue;
}

/**
 * Transition hook for marking updates as non-urgent
 */
export function useTransition(): [boolean, (callback: () => void) => void] {
    const [isPending, setIsPending] = useState(false);
    
    const startTransition = useCallback((callback: () => void) => {
        setIsPending(true);
        
        setTimeout(() => {
            callback();
            setIsPending(false);
        }, 0);
    }, []);
    
    return [isPending, startTransition];
}

/**
 * Sync external store hook for subscribing to external data sources
 */
export function useSyncExternalStore<T>(
    subscribe: (callback: () => void) => () => void,
    getSnapshot: () => T,
    getServerSnapshot?: () => T
): T {
    const [value, setValue] = useState(getSnapshot);
    
    useEffect(() => {
        const handleChange = () => {
            setValue(getSnapshot());
        };
        
        const unsubscribe = subscribe(handleChange);
        
        // Check for changes that happened between render and effect
        handleChange();
        
        return unsubscribe;
    }, [subscribe, getSnapshot]);
    
    return value;
}