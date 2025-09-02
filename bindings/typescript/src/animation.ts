/**
 * Animation API
 */

import { lib, VoidPtr } from './ffi';
import { checkError } from './error';

export enum AnimationType {
  Linear = 0,
  EaseIn = 1,
  EaseOut = 2,
  EaseInOut = 3,
  Bounce = 4,
  Elastic = 5,
  Spring = 6,
}

export enum AnimationProperty {
  X = 0,
  Y = 1,
  Width = 2,
  Height = 3,
  Opacity = 4,
  Color = 5,
  All = 6,
}

export interface AnimationOptions {
  duration?: number;
  type?: AnimationType;
  delay?: number;
  repeat?: number;
  reverse?: boolean;
  onComplete?: () => void;
  onProgress?: (progress: number) => void;
}

export class Animation {
  private handle: Buffer;
  private callbacks: Map<string, Function> = new Map();

  constructor(property: AnimationProperty, from: number, to: number, options?: AnimationOptions) {
    const opts = options || {};
    const duration = opts.duration || 300;
    const type = opts.type !== undefined ? opts.type : AnimationType.Linear;
    const delay = opts.delay || 0;
    const repeat = opts.repeat || 1;
    const reverse = opts.reverse || false;

    this.handle = lib.rtui_animation_create(property, from, to, duration, type, delay, repeat, reverse);
    
    if (this.handle.isNull()) {
      throw new Error(`Failed to create animation for property ${AnimationProperty[property]}`);
    }

    if (opts.onComplete) {
      this.callbacks.set('complete', opts.onComplete);
    }
    if (opts.onProgress) {
      this.callbacks.set('progress', opts.onProgress);
    }
  }

  /**
   * Start the animation
   */
  start(): void {
    checkError(lib.rtui_animation_start(this.handle));
  }

  /**
   * Pause the animation
   */
  pause(): void {
    checkError(lib.rtui_animation_pause(this.handle));
  }

  /**
   * Resume a paused animation
   */
  resume(): void {
    checkError(lib.rtui_animation_resume(this.handle));
  }

  /**
   * Stop the animation
   */
  stop(): void {
    checkError(lib.rtui_animation_stop(this.handle));
  }

  /**
   * Reset the animation to the beginning
   */
  reset(): void {
    checkError(lib.rtui_animation_reset(this.handle));
  }

  /**
   * Get the current progress (0.0 to 1.0)
   */
  getProgress(): number {
    return lib.rtui_animation_get_progress(this.handle);
  }

  /**
   * Set the current progress (0.0 to 1.0)
   */
  setProgress(progress: number): void {
    checkError(lib.rtui_animation_set_progress(this.handle, progress));
  }

  /**
   * Check if the animation is running
   */
  isRunning(): boolean {
    return lib.rtui_animation_is_running(this.handle) !== 0;
  }

  /**
   * Check if the animation is complete
   */
  isComplete(): boolean {
    return lib.rtui_animation_is_complete(this.handle) !== 0;
  }

  /**
   * Update the animation (typically called in a render loop)
   */
  update(deltaTime: number): void {
    checkError(lib.rtui_animation_update(this.handle, deltaTime));
    
    // Check for callbacks
    const progress = this.getProgress();
    const progressCallback = this.callbacks.get('progress');
    if (progressCallback) {
      progressCallback(progress);
    }
    
    if (this.isComplete()) {
      const completeCallback = this.callbacks.get('complete');
      if (completeCallback) {
        completeCallback();
      }
    }
  }

  /**
   * Free the animation resources
   */
  dispose(): void {
    if (!this.handle.isNull()) {
      lib.rtui_animation_free(this.handle);
      this.handle = Buffer.alloc(0);
      this.callbacks.clear();
    }
  }

  /**
   * Get the native handle for low-level operations
   */
  getNativeHandle(): Buffer {
    return this.handle;
  }
}

/**
 * Animation group for coordinating multiple animations
 */
export class AnimationGroup {
  private handle: Buffer;
  private animations: Animation[] = [];

  constructor() {
    this.handle = lib.rtui_animation_group_create();
    
    if (this.handle.isNull()) {
      throw new Error('Failed to create animation group');
    }
  }

  /**
   * Add an animation to the group
   */
  add(animation: Animation): void {
    checkError(lib.rtui_animation_group_add(this.handle, animation.getNativeHandle()));
    this.animations.push(animation);
  }

  /**
   * Remove an animation from the group
   */
  remove(animation: Animation): void {
    checkError(lib.rtui_animation_group_remove(this.handle, animation.getNativeHandle()));
    const index = this.animations.indexOf(animation);
    if (index !== -1) {
      this.animations.splice(index, 1);
    }
  }

  /**
   * Start all animations in the group
   */
  start(): void {
    checkError(lib.rtui_animation_group_start(this.handle));
  }

  /**
   * Stop all animations in the group
   */
  stop(): void {
    checkError(lib.rtui_animation_group_stop(this.handle));
  }

  /**
   * Pause all animations in the group
   */
  pause(): void {
    checkError(lib.rtui_animation_group_pause(this.handle));
  }

  /**
   * Resume all animations in the group
   */
  resume(): void {
    checkError(lib.rtui_animation_group_resume(this.handle));
  }

  /**
   * Update all animations in the group
   */
  update(deltaTime: number): void {
    checkError(lib.rtui_animation_group_update(this.handle, deltaTime));
  }

  /**
   * Check if any animation is running
   */
  isRunning(): boolean {
    return lib.rtui_animation_group_is_running(this.handle) !== 0;
  }

  /**
   * Free the animation group resources
   */
  dispose(): void {
    if (!this.handle.isNull()) {
      lib.rtui_animation_group_free(this.handle);
      this.handle = Buffer.alloc(0);
      this.animations = [];
    }
  }
}

/**
 * Helper function to create a fade-in animation
 */
export function fadeIn(duration: number = 300): Animation {
  return new Animation(AnimationProperty.Opacity, 0, 1, {
    duration,
    type: AnimationType.EaseIn,
  });
}

/**
 * Helper function to create a fade-out animation
 */
export function fadeOut(duration: number = 300): Animation {
  return new Animation(AnimationProperty.Opacity, 1, 0, {
    duration,
    type: AnimationType.EaseOut,
  });
}

/**
 * Helper function to create a slide animation
 */
export function slide(fromX: number, toX: number, duration: number = 300): Animation {
  return new Animation(AnimationProperty.X, fromX, toX, {
    duration,
    type: AnimationType.EaseInOut,
  });
}

/**
 * Helper function to create a bounce animation
 */
export function bounce(property: AnimationProperty, from: number, to: number, duration: number = 500): Animation {
  return new Animation(property, from, to, {
    duration,
    type: AnimationType.Bounce,
  });
}

/**
 * Helper function to create a spring animation
 */
export function spring(property: AnimationProperty, from: number, to: number, duration: number = 400): Animation {
  return new Animation(property, from, to, {
    duration,
    type: AnimationType.Spring,
  });
}