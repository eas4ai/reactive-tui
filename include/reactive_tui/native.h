#ifndef REACTIVE_TUI_NATIVE_H
#define REACTIVE_TUI_NATIVE_H

/* Generated from Rust by cbindgen 0.29.4. Run scripts/generate-native-header.py. */

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
typedef struct RTuiRenderer RTuiRenderer;
typedef struct RTuiBuffer RTuiBuffer;
typedef struct RTuiTerminal RTuiTerminal;
typedef struct RTuiTextBuffer RTuiTextBuffer;
typedef struct RTuiAnimationManager RTuiAnimationManager;
typedef struct RTuiAnimation RTuiAnimation;
typedef struct RTuiAppBuilder RTuiAppBuilder;
typedef struct RTuiElement RTuiElement;
typedef struct RTuiApp RTuiApp;
typedef struct RTuiElementBuilder RTuiElementBuilder;
typedef struct RTuiSignal RTuiSignal;
typedef struct RTuiThreadSafeSignal RTuiThreadSafeSignal;
typedef struct RTuiEffect RTuiEffect;
typedef struct RTuiHooks RTuiHooks;
typedef struct RTuiSurface RTuiSurface;
typedef struct RTuiTextEditor RTuiTextEditor;
typedef struct RTuiNativeStyle RTuiNativeStyle;
typedef struct RTuiDialogEngine RTuiDialogEngine;
typedef struct RTuiForeignComponent RTuiForeignComponent;

/* Signal ownership: each constructor and hook lookup returns an owning handle.
 * Release it exactly once with either rtui_signal_destroy or
 * rtui_signal_destroy_new. Handles are confined to their creating thread;
 * use RTuiThreadSafeSignal only with its separate functions/destructor.
 * Both RTuiSignal function families use the same typed allocation. String,
 * boolean and float access interoperates; legacy int is int64_t while the
 * improved int is C int. Wrong live types return InvalidParameter (or the
 * documented getter default). Arbitrary/stale pointers are invalid.
 * Owned getter strings require rtui_string_free independently of the signal.
 *
 * Snapshot arrays returned by bufferGet*Ptr remain owned by the library until
 * their matching bufferRelease*Ptr call. The library retains their allocation
 * metadata; the length argument is accepted for ABI compatibility and does not
 * control deallocation. Interior, misaligned, and repeated releases are ignored.
 *
 * createOptimizedBuffer and createTextBuffer return tracked owning handles.
 * Their destroy functions ignore NULL and already-destroyed handles. AppBuilder
 * build consumes every non-NULL builder, including on failure. Animation-manager
 * add consumes the animation on success. rtui_app_run borrows its retained App
 * handle while it blocks; another thread may call rtui_app_quit, and the owner
 * destroys the handle only after run returns. Optional root and effect-cleanup
 * callbacks may be NULL.
 */


typedef enum RTuiError {
  R_TUI_ERROR_SUCCESS = 0,
  R_TUI_ERROR_INVALID_PARAMETER = -1,
  R_TUI_ERROR_NULL_POINTER = -2,
  R_TUI_ERROR_BUFFER_TOO_SMALL = -3,
  R_TUI_ERROR_OUT_OF_MEMORY = -4,
  R_TUI_ERROR_INVALID_UTF8 = -5,
  R_TUI_ERROR_TERMINAL_NOT_AVAILABLE = -6,
  R_TUI_ERROR_NOT_SUPPORTED = -7,
  R_TUI_ERROR_ALREADY_EXISTS = -8,
  R_TUI_ERROR_NOT_FOUND = -9,
  R_TUI_ERROR_INVALID_STATE = -10,
  R_TUI_ERROR_INVALID_POINTER = -11,
  R_TUI_ERROR_INTERNAL_ERROR = -12,
  R_TUI_ERROR_PANIC = -99,
  R_TUI_ERROR_UNKNOWN = -100,
} RTuiError;

typedef enum RTuiEventType {
  R_TUI_EVENT_TYPE_KEY = 0,
  R_TUI_EVENT_TYPE_MOUSE = 1,
  R_TUI_EVENT_TYPE_RESIZE = 2,
  R_TUI_EVENT_TYPE_FOCUS = 3,
  R_TUI_EVENT_TYPE_PASTE = 4,
} RTuiEventType;

typedef enum RTuiEasingType {
  R_TUI_EASING_TYPE_LINEAR = 0,
  R_TUI_EASING_TYPE_EASE_IN = 1,
  R_TUI_EASING_TYPE_EASE_OUT = 2,
  R_TUI_EASING_TYPE_EASE_IN_OUT = 3,
  R_TUI_EASING_TYPE_BOUNCE = 4,
  R_TUI_EASING_TYPE_ELASTIC = 5,
  R_TUI_EASING_TYPE_BACK = 6,
  R_TUI_EASING_TYPE_EXPO = 7,
  R_TUI_EASING_TYPE_CIRC = 8,
  R_TUI_EASING_TYPE_SINE = 9,
  R_TUI_EASING_TYPE_QUAD = 10,
  R_TUI_EASING_TYPE_CUBIC = 11,
  R_TUI_EASING_TYPE_QUART = 12,
  R_TUI_EASING_TYPE_QUINT = 13,
  R_TUI_EASING_TYPE_SPRING = 14,
} RTuiEasingType;

typedef enum RTuiLoopMode {
  R_TUI_LOOP_MODE_NONE = 0,
  R_TUI_LOOP_MODE_INFINITE = 1,
  R_TUI_LOOP_MODE_COUNT = 2,
  R_TUI_LOOP_MODE_PING_PONG = 3,
} RTuiLoopMode;

typedef enum RTuiAnimatedProperty {
  R_TUI_ANIMATED_PROPERTY_OPACITY = 0,
  R_TUI_ANIMATED_PROPERTY_TRANSLATE_X = 1,
  R_TUI_ANIMATED_PROPERTY_TRANSLATE_Y = 2,
  R_TUI_ANIMATED_PROPERTY_SCALE_X = 3,
  R_TUI_ANIMATED_PROPERTY_SCALE_Y = 4,
  R_TUI_ANIMATED_PROPERTY_ROTATION = 5,
  R_TUI_ANIMATED_PROPERTY_WIDTH = 6,
  R_TUI_ANIMATED_PROPERTY_HEIGHT = 7,
} RTuiAnimatedProperty;

typedef enum RTuiPerformanceMode {
  R_TUI_PERFORMANCE_MODE_POWER_SAVE = 0,
  R_TUI_PERFORMANCE_MODE_BALANCED = 1,
  R_TUI_PERFORMANCE_MODE_PERFORMANCE = 2,
} RTuiPerformanceMode;

typedef enum RTuiLayoutType {
  R_TUI_LAYOUT_TYPE_FLEX = 0,
  R_TUI_LAYOUT_TYPE_GRID = 1,
  R_TUI_LAYOUT_TYPE_STACK = 2,
  R_TUI_LAYOUT_TYPE_ABSOLUTE = 3,
} RTuiLayoutType;

typedef enum RTuiElementType {
  R_TUI_ELEMENT_TYPE_COMPONENT = 0,
  R_TUI_ELEMENT_TYPE_TEXT = 1,
  R_TUI_ELEMENT_TYPE_LAYOUT = 2,
  R_TUI_ELEMENT_TYPE_FRAGMENT = 3,
  R_TUI_ELEMENT_TYPE_EMPTY = 4,
} RTuiElementType;

typedef struct RTuiVersion {
  uint32_t major;
  uint32_t minor;
  uint32_t patch;
  uint32_t abi_version;
} RTuiVersion;

typedef struct RTuiCapabilities {
  bool rgb;
  bool color_256;
  uint8_t unicode_level;
  bool kitty_keyboard;
  bool mouse;
  bool pixel_mouse;
  bool hyperlinks;
  bool images;
  bool synchronized_output;
  bool bracketed_paste;
} RTuiCapabilities;

typedef struct RTuiDimensions {
  uint16_t width;
  uint16_t height;
} RTuiDimensions;

typedef struct RTuiKeyEvent {
  uint32_t key_code;
  uint8_t modifiers;
} RTuiKeyEvent;

typedef struct RTuiMouseEvent {
  uint16_t x;
  uint16_t y;
  uint8_t button;
  uint8_t modifiers;
  uint8_t event_type;
} RTuiMouseEvent;

typedef struct RTuiResizeEvent {
  uint16_t width;
  uint16_t height;
} RTuiResizeEvent;

typedef union RTuiEventData {
  struct RTuiKeyEvent key;
  struct RTuiMouseEvent mouse;
  struct RTuiResizeEvent resize;
} RTuiEventData;

typedef struct RTuiEvent {
  enum RTuiEventType event_type;
  union RTuiEventData data;
} RTuiEvent;

typedef RTuiElement *(*RTuiRootComponentCallback)(void *user_data);

typedef struct RTuiPerformanceMetrics {
  float current_fps;
  float avg_render_time_ms;
  float drop_rate_percent;
  bool is_stable;
} RTuiPerformanceMetrics;

typedef int32_t (*RTuiForeignRenderCallback)(const char*, const char*, void*, RTuiElement**);

typedef int32_t (*RTuiForeignEventCallback)(const char*, const char*, const char*, void*, bool*);

typedef void (*RTuiForeignDisposeCallback)(void*);

typedef void (*RTuiEffectCallback)(void *user_data);

typedef void (*RTuiEffectCleanupCallback)(void *user_data);

typedef struct RTuiColor {
  uint8_t r;
  uint8_t g;
  uint8_t b;
} RTuiColor;

typedef struct RTuiTextAttributes {
  bool bold;
  bool italic;
  bool underline;
  bool strikethrough;
  bool reverse;
  bool blink;
  bool hidden;
} RTuiTextAttributes;

typedef struct RTuiCell {
  uint32_t ch;
  struct RTuiColor fg;
  struct RTuiColor bg;
  struct RTuiTextAttributes attrs;
} RTuiCell;

typedef struct RTuiRect {
  uint16_t x;
  uint16_t y;
  uint16_t width;
  uint16_t height;
} RTuiRect;

typedef void (*LogCallback)(uint8_t level, const uint8_t *msg_ptr, size_t msg_len);

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

struct RTuiVersion rtui_version(void);

enum RTuiError rtui_init(void);

void rtui_cleanup(void);

RTuiRenderer *createRenderer(uint32_t width, uint32_t height);

void destroyRenderer(RTuiRenderer *renderer, bool _use_alternate_screen, uint32_t _split_height);

void setBackgroundColor(RTuiRenderer *renderer, const float *color);

void render(RTuiRenderer *renderer, bool force);

void resizeRenderer(RTuiRenderer *renderer, uint32_t width, uint32_t height);

RTuiBuffer *createOptimizedBuffer(uint32_t width,
                                  uint32_t height,
                                  bool _respect_alpha,
                                  uint8_t _width_method,
                                  const uint8_t *_id_ptr,
                                  size_t _id_len);

void destroyOptimizedBuffer(RTuiBuffer *buffer);

uint32_t getBufferWidth(const RTuiBuffer *buffer);

uint32_t getBufferHeight(const RTuiBuffer *buffer);

void bufferClear(RTuiBuffer *buffer, const float *bg);

void bufferDrawText(RTuiBuffer *buffer,
                    const uint8_t *text,
                    size_t text_len,
                    uint32_t x,
                    uint32_t y,
                    const float *fg,
                    const float *bg,
                    uint8_t attributes);

void bufferSetCellWithAlphaBlending(RTuiBuffer *buffer,
                                    uint32_t x,
                                    uint32_t y,
                                    uint32_t character,
                                    const float *fg,
                                    const float *bg,
                                    uint8_t attributes);

void bufferFillRect(RTuiBuffer *buffer,
                    uint32_t x,
                    uint32_t y,
                    uint32_t width,
                    uint32_t height,
                    const float *bg);

uint32_t *bufferGetCharPtr(RTuiBuffer *buffer);

void bufferReleaseCharPtr(uint32_t *ptr, size_t _length);

float *bufferGetFgPtr(RTuiBuffer *buffer);

void bufferReleaseFgPtr(float *ptr, size_t _length);

float *bufferGetBgPtr(RTuiBuffer *buffer);

void bufferReleaseBgPtr(float *ptr, size_t _length);

uint8_t *bufferGetAttributesPtr(RTuiBuffer *buffer);

void bufferReleaseAttrPtr(uint8_t *ptr, size_t _length);

bool bufferGetRespectAlpha(const RTuiBuffer *buffer);

void bufferSetRespectAlpha(RTuiBuffer *buffer, bool respect_alpha);

void bufferResize(RTuiBuffer *buffer, uint32_t width, uint32_t height);

bool renderSurfaceToTerminal(const RTuiBuffer *surface, RTuiTerminal *terminal);

bool renderTextToTerminal(const RTuiTextBuffer *text_buffer,
                          RTuiTerminal *terminal,
                          uint32_t x,
                          uint32_t y,
                          uint32_t width,
                          uint32_t height);

bool renderWithStats(RTuiRenderer *renderer, RTuiTerminal *terminal, bool collect_stats);

void updateStats(RTuiRenderer *renderer, double _time, uint32_t _fps, double _frame_callback_time);

void updateMemoryStats(RTuiRenderer *renderer,
                       uint32_t _heap_used,
                       uint32_t _heap_total,
                       uint32_t _array_buffers);

void setRenderOffset(RTuiRenderer *renderer, uint32_t offset);

void setDebugOverlay(RTuiRenderer *renderer, bool enabled, uint8_t _corner);

void addToHitGrid(RTuiRenderer *renderer,
                  int32_t _x,
                  int32_t _y,
                  uint32_t _width,
                  uint32_t _height,
                  uint32_t _id);

uint32_t checkHit(RTuiRenderer *renderer, uint32_t x, uint32_t y);

void dumpHitGrid(RTuiRenderer *renderer);

void dumpBuffers(RTuiRenderer *renderer, int64_t timestamp);

void dumpStdoutBuffer(RTuiRenderer *renderer, int64_t timestamp);

void setLogCallback(void (*callback)(uint8_t level, const uint8_t *msg_ptr, size_t msg_len));

void startProfiling(RTuiRenderer *renderer);

void stopProfiling(RTuiRenderer *renderer);

void getFrameStats(const RTuiRenderer *renderer,
                   float *out_avg_frame_time,
                   float *out_min_frame_time,
                   float *out_max_frame_time,
                   uint32_t *out_frame_count);

void resetPerformanceCounters(RTuiRenderer *renderer);

RTuiTerminal *createTerminal(void);

void destroyTerminal(RTuiTerminal *terminal);

void setupTerminal(RTuiTerminal *terminal, bool use_alternate_screen);

void clearTerminal(RTuiTerminal *terminal);

void getTerminalCapabilities(const RTuiTerminal *terminal, struct RTuiCapabilities *caps_ptr);

void processCapabilityResponse(RTuiTerminal *terminal,
                               const uint8_t *response_ptr,
                               size_t response_len);

void setCursorPosition(RTuiTerminal *terminal, int32_t x, int32_t y, bool visible);

void setCursorStyle(RTuiTerminal *terminal,
                    const uint8_t *style_ptr,
                    size_t style_len,
                    bool blinking);

void setCursorColor(RTuiTerminal *terminal, const float *color);

void setTerminalTitle(RTuiTerminal *terminal, const uint8_t *title_ptr, size_t title_len);

void enableMouse(RTuiTerminal *terminal, bool enable_movement);

void disableMouse(RTuiTerminal *terminal);

void enableKittyKeyboard(RTuiTerminal *terminal, uint8_t flags);

void disableKittyKeyboard(RTuiTerminal *terminal);

enum RTuiError rtui_terminal_create(RTuiTerminal **out_terminal);

void rtui_terminal_destroy(RTuiTerminal *terminal);

enum RTuiError rtui_terminal_get_dimensions(const RTuiTerminal *terminal,
                                            struct RTuiDimensions *out_dimensions);

enum RTuiError rtui_terminal_sync(RTuiTerminal *terminal, bool begin);

enum RTuiError rtui_terminal_poll_event(uint32_t timeout_ms, struct RTuiEvent *out_event);

RTuiTextBuffer *createTextBuffer(uint32_t length, uint8_t _width_method);

void destroyTextBuffer(RTuiTextBuffer *tb);

uint32_t *textBufferGetCharPtr(RTuiTextBuffer *tb);

uint32_t textBufferGetLength(const RTuiTextBuffer *tb);

uint32_t textBufferGetCapacity(const RTuiTextBuffer *tb);

void textBufferResize(RTuiTextBuffer *tb, uint32_t new_length);

void textBufferReset(RTuiTextBuffer *tb);

uint32_t textBufferWriteChunk(RTuiTextBuffer *tb,
                              const uint8_t *text_bytes,
                              uint32_t text_len,
                              const float *fg,
                              const float *bg,
                              const uint8_t *attr);

void textBufferSetSelection(RTuiTextBuffer *tb,
                            uint32_t start,
                            uint32_t end,
                            const float *bg_color,
                            const float *fg_color);

uint32_t renderTextBufferToSurface(const RTuiTextBuffer *tb,
                                   RTuiBuffer *buffer,
                                   uint32_t x,
                                   uint32_t y,
                                   uint32_t max_width);

uint32_t renderTextBufferToRenderer(const RTuiTextBuffer *tb,
                                    RTuiRenderer *renderer,
                                    uint32_t x,
                                    uint32_t y,
                                    uint32_t max_width);

bool renderTextBufferDirect(const RTuiTextBuffer *tb,
                            RTuiTerminal *terminal,
                            uint32_t x,
                            uint32_t y,
                            uint32_t width,
                            uint32_t height);

void textBufferResetSelection(RTuiTextBuffer *tb);

uint64_t textBufferGetSelectionInfo(const RTuiTextBuffer *tb);

void textBufferSetDefaultFg(RTuiTextBuffer *tb, const float *fg);

void textBufferSetDefaultBg(RTuiTextBuffer *tb, const float *bg);

void textBufferSetDefaultAttributes(RTuiTextBuffer *tb, const uint8_t *attr);

void textBufferResetDefaults(RTuiTextBuffer *tb);

enum RTuiError rtui_animation_manager_create(RTuiAnimationManager **out_manager);

void rtui_animation_manager_destroy(RTuiAnimationManager *manager);

enum RTuiError rtui_animation_manager_update(RTuiAnimationManager *manager);

enum RTuiError rtui_animation_create(const char *id,
                                     uint32_t duration_ms,
                                     enum RTuiEasingType easing,
                                     enum RTuiLoopMode loop_mode,
                                     uint32_t loop_count,
                                     RTuiAnimation **out_animation);

void rtui_animation_destroy(RTuiAnimation *animation);

enum RTuiError rtui_animation_set_property(RTuiAnimation *animation,
                                           enum RTuiAnimatedProperty property,
                                           float from_value,
                                           float to_value);

enum RTuiError rtui_animation_manager_add(RTuiAnimationManager *manager,
                                          RTuiAnimation *animation,
                                          char **out_id);

enum RTuiError rtui_animation_manager_remove(RTuiAnimationManager *manager,
                                             const char *animation_id);

enum RTuiError rtui_animation_play(RTuiAnimation *animation);

enum RTuiError rtui_animation_pause(RTuiAnimation *animation);

enum RTuiError rtui_animation_stop(RTuiAnimation *animation);

enum RTuiError rtui_animation_is_playing(const RTuiAnimation *animation, bool *out_playing);

enum RTuiError rtui_animation_get_progress(const RTuiAnimation *animation, float *out_progress);

enum RTuiError rtui_animation_create_spring(const char *id,
                                            float stiffness,
                                            float damping,
                                            float mass,
                                            RTuiAnimation **out_animation);

enum RTuiError rtui_app_builder_root_element(RTuiAppBuilder *builder, RTuiElement *element);

enum RTuiError rtui_app_builder_create(RTuiAppBuilder **out_builder);

void rtui_app_builder_destroy(RTuiAppBuilder *builder);

enum RTuiError rtui_app_builder_debug(RTuiAppBuilder *builder, bool debug);

enum RTuiError rtui_app_builder_performance_mode(RTuiAppBuilder *builder,
                                                 enum RTuiPerformanceMode mode);

enum RTuiError rtui_app_builder_backend_debug(RTuiAppBuilder *builder,
                                              uint16_t width,
                                              uint16_t height);

enum RTuiError rtui_app_builder_backend_suprtui(RTuiAppBuilder *builder);

enum RTuiError rtui_app_builder_backend_crossterm(RTuiAppBuilder *builder);

enum RTuiError rtui_app_builder_root_component(RTuiAppBuilder *builder,
                                               RTuiRootComponentCallback callback,
                                               void *user_data);

enum RTuiError rtui_app_builder_build(RTuiAppBuilder *builder, RTuiApp **out_app);

void rtui_app_destroy(RTuiApp *app);

enum RTuiError rtui_app_run(RTuiApp *app);

enum RTuiError rtui_app_quit(RTuiApp *app);

enum RTuiError rtui_app_get_size(const RTuiApp *app, struct RTuiDimensions *out_dimensions);

enum RTuiError rtui_app_set_performance_mode(RTuiApp *app, enum RTuiPerformanceMode mode);

enum RTuiError rtui_app_get_current_fps(const RTuiApp *app, uint32_t *out_fps);

enum RTuiError rtui_app_get_performance_metrics(const RTuiApp *app,
                                                struct RTuiPerformanceMetrics *out_metrics);

enum RTuiError rtui_element_builder_div(RTuiElementBuilder **out_builder);

enum RTuiError rtui_element_builder_span(RTuiElementBuilder **out_builder);

enum RTuiError rtui_element_builder_button(RTuiElementBuilder **out_builder);

enum RTuiError rtui_element_builder_add_class(RTuiElementBuilder *builder, const char *classes);

enum RTuiError rtui_element_builder_set_text(RTuiElementBuilder *builder, const char *text);

enum RTuiError rtui_element_builder_set_key(RTuiElementBuilder *builder, const char *key);

enum RTuiError rtui_element_builder_add_child(RTuiElementBuilder *builder, RTuiElement *child);

enum RTuiError rtui_element_builder_build(RTuiElementBuilder *builder, RTuiElement **out_element);

void rtui_element_builder_destroy(RTuiElementBuilder *builder);

void rtui_element_destroy(RTuiElement *element);

enum RTuiError rtui_div(RTuiElementBuilder **out);

enum RTuiError rtui_span(RTuiElementBuilder **out);

enum RTuiError rtui_button(RTuiElementBuilder **out);

enum RTuiError rtui_p(RTuiElementBuilder **out);

enum RTuiError rtui_h1(RTuiElementBuilder **out);

enum RTuiError rtui_h2(RTuiElementBuilder **out);

enum RTuiError rtui_h3(RTuiElementBuilder **out);

enum RTuiError rtui_element_builder_class(RTuiElementBuilder *builder, const char *classes);

enum RTuiError rtui_element_builder_text(RTuiElementBuilder *builder, const char *text);

enum RTuiError rtui_element_builder_key(RTuiElementBuilder *builder, const char *key);

enum RTuiError rtui_element_builder_child(RTuiElementBuilder *builder, RTuiElement *child);

enum RTuiError rtui_element_builder_children(RTuiElementBuilder *builder,
                                             RTuiElement *const *children,
                                             size_t children_count);

enum RTuiError rtui_element_text(const char *text, RTuiElement **out);

enum RTuiError rtui_element_empty(RTuiElement **out);

enum RTuiError rtui_card(RTuiElement *const *children, size_t children_count, RTuiElement **out);

void rtui_free_string(char *string);

enum RTuiError rtui_element_create_component(const char *name, RTuiElement **out_element);

enum RTuiError rtui_element_create_text(const char *text, RTuiElement **out_element);

enum RTuiError rtui_element_create_layout(enum RTuiLayoutType layout_type,
                                          RTuiElement **out_element);

enum RTuiError rtui_element_create_fragment(RTuiElement **out_element);

enum RTuiError rtui_element_create_empty(RTuiElement **out_element);

enum RTuiError rtui_element_set_key(RTuiElement *element, const char *key);

enum RTuiError rtui_element_set_class(RTuiElement *element, const char *class_);

enum RTuiError rtui_element_add_child(RTuiElement *parent, RTuiElement *child);

enum RTuiError rtui_element_get_type(const RTuiElement *element, enum RTuiElementType *out_type);

enum RTuiError rtui_element_get_key(const RTuiElement *element, char **out_key);

enum RTuiError rtui_element_get_class(const RTuiElement *element, char **out_class);

enum RTuiError rtui_element_get_child_count(const RTuiElement *element, size_t *out_count);

enum RTuiError rtui_element_get_child(const RTuiElement *element,
                                      size_t index,
                                      RTuiElement **out_child);

enum RTuiError rtui_element_get_component_name(const RTuiElement *element, char **out_name);

enum RTuiError rtui_element_get_text_content(const RTuiElement *element, char **out_text);

enum RTuiError rtui_dialog_engine_create(RTuiDialogEngine **out_engine);

enum RTuiError rtui_dialog_engine_destroy(RTuiDialogEngine *engine);

enum RTuiError rtui_dialog_engine_element(const RTuiDialogEngine *engine,
                                          RTuiElement **out_element);

enum RTuiError rtui_dialog_engine_open(RTuiDialogEngine *engine,
                                       const char *options,
                                       uint32_t *out_id);

enum RTuiError rtui_dialog_engine_update(RTuiDialogEngine *engine, uint32_t id, const char *update);

enum RTuiError rtui_dialog_engine_close(RTuiDialogEngine *engine, uint32_t id, const char *result);

enum RTuiError rtui_dialog_engine_take_event(RTuiDialogEngine *engine, char **out_event);

enum RTuiError rtui_text_editor_create(RTuiTextEditor **out_editor);

enum RTuiError rtui_text_editor_destroy(RTuiTextEditor *editor);

enum RTuiError rtui_text_editor_set_content(RTuiTextEditor *editor, const char *content);

enum RTuiError rtui_text_editor_get_content_owned(const RTuiTextEditor *editor, char **out_content);

enum RTuiError rtui_text_editor_insert_text(RTuiTextEditor *editor, const char *text);

enum RTuiError rtui_text_editor_delete(RTuiTextEditor *editor, bool backward);

enum RTuiError rtui_text_editor_move(RTuiTextEditor *editor, uint32_t movement, bool select);

enum RTuiError rtui_text_editor_set_size(RTuiTextEditor *editor, uint32_t width, uint32_t height);

enum RTuiError rtui_text_editor_set_show_line_numbers(RTuiTextEditor *editor, bool show);

enum RTuiError rtui_text_editor_element(const RTuiTextEditor *editor, RTuiElement **out_element);

enum RTuiError rtui_foreign_component_create(const char *props,
                                             const char *state,
                                             RTuiForeignRenderCallback render,
                                             RTuiForeignEventCallback event,
                                             RTuiForeignDisposeCallback dispose,
                                             void *userdata,
                                             RTuiForeignComponent **out_component);

enum RTuiError rtui_foreign_component_destroy(RTuiForeignComponent *component);

enum RTuiError rtui_foreign_component_element(const RTuiForeignComponent *component,
                                              RTuiElement **out_element);

enum RTuiError rtui_foreign_component_render(const RTuiForeignComponent *component,
                                             RTuiElement **out_element);

enum RTuiError rtui_foreign_component_dispatch(const RTuiForeignComponent *component,
                                               const char *event,
                                               bool *out_handled);

enum RTuiError rtui_foreign_component_get_props(const RTuiForeignComponent *component,
                                                char **out_value);

enum RTuiError rtui_foreign_component_get_state(const RTuiForeignComponent *component,
                                                char **out_value);

enum RTuiError rtui_foreign_component_set_props(const RTuiForeignComponent *component,
                                                const char *value);

enum RTuiError rtui_foreign_component_set_state(const RTuiForeignComponent *component,
                                                const char *value);

enum RTuiError rtui_element_set_focus(RTuiElement *element, bool focusable, bool auto_focus);

enum RTuiError rtui_foreign_component_last_error(const RTuiForeignComponent *component,
                                                 int32_t *out_code);

enum RTuiError rtui_native_style_create(const char *css, RTuiNativeStyle **out_style);

enum RTuiError rtui_native_style_apply(const RTuiNativeStyle *style, RTuiElement *element);

enum RTuiError rtui_native_style_destroy(RTuiNativeStyle *style);

enum RTuiError rtui_signal_string_create(const char *initial_value, RTuiSignal **out_signal);

enum RTuiError rtui_signal_int_create(int64_t initial_value, RTuiSignal **out_signal);

enum RTuiError rtui_signal_float_create(double initial_value, RTuiSignal **out_signal);

enum RTuiError rtui_signal_bool_create(bool initial_value, RTuiSignal **out_signal);

void rtui_signal_destroy(RTuiSignal *signal);

enum RTuiError rtui_signal_string_get(const RTuiSignal *signal, char *buffer, size_t buffer_size);

enum RTuiError rtui_signal_string_set(RTuiSignal *signal, const char *value);

enum RTuiError rtui_signal_int_get(const RTuiSignal *signal, int64_t *out_value);

enum RTuiError rtui_signal_int_set(RTuiSignal *signal, int64_t value);

enum RTuiError rtui_signal_float_get(const RTuiSignal *signal, double *out_value);

enum RTuiError rtui_signal_float_set(RTuiSignal *signal, double value);

enum RTuiError rtui_signal_bool_get(const RTuiSignal *signal, bool *out_value);

enum RTuiError rtui_signal_bool_set(RTuiSignal *signal, bool value);

enum RTuiError rtui_thread_safe_signal_string_create(const char *initial_value,
                                                     RTuiThreadSafeSignal **out_signal);

void rtui_thread_safe_signal_destroy(RTuiThreadSafeSignal *signal);

enum RTuiError rtui_thread_safe_signal_string_get(const RTuiThreadSafeSignal *signal,
                                                  char *buffer,
                                                  size_t buffer_size);

enum RTuiError rtui_thread_safe_signal_string_set(RTuiThreadSafeSignal *signal, const char *value);

enum RTuiError rtui_effect_create(RTuiEffectCallback callback,
                                  RTuiEffectCleanupCallback cleanup,
                                  void *user_data,
                                  RTuiEffect **out_effect);

void rtui_effect_destroy(RTuiEffect *effect);

enum RTuiError rtui_effect_run(const RTuiEffect *effect);

RTuiSignal *rtui_signal_new_int(int initial_value);

RTuiSignal *rtui_signal_new_string(const char *initial_value);

RTuiSignal *rtui_signal_new_bool(bool initial_value);

RTuiSignal *rtui_signal_new_float(double initial_value);

int rtui_signal_get_int(const RTuiSignal *signal);

enum RTuiError rtui_signal_set_int(RTuiSignal *signal, int value);

char *rtui_signal_get_string_owned(const RTuiSignal *signal);

enum RTuiError rtui_signal_set_string_new(RTuiSignal *signal, const char *value);

bool rtui_signal_get_bool_new(const RTuiSignal *signal);

enum RTuiError rtui_signal_set_bool_new(RTuiSignal *signal, bool value);

double rtui_signal_get_float_new(const RTuiSignal *signal);

enum RTuiError rtui_signal_set_float_new(RTuiSignal *signal, double value);

void rtui_signal_destroy_new(RTuiSignal *signal);

void rtui_string_free(char *string);

RTuiHooks *rtui_hooks_new(void);

void rtui_hooks_destroy(RTuiHooks *hooks);

RTuiSignal *rtui_use_signal_int(RTuiHooks *hooks, const char *key, int initial);

RTuiSignal *rtui_use_signal_string(RTuiHooks *hooks, const char *key, const char *initial);

RTuiSignal *rtui_use_signal_bool(RTuiHooks *hooks, const char *key, bool initial);

enum RTuiError rtui_renderer_create(uint16_t width, uint16_t height, RTuiRenderer **out_renderer);

void rtui_renderer_destroy(RTuiRenderer *renderer);

enum RTuiError rtui_renderer_resize(RTuiRenderer *renderer, uint16_t width, uint16_t height);

enum RTuiError rtui_renderer_clear(RTuiRenderer *renderer, uint8_t r, uint8_t g, uint8_t b);

enum RTuiError rtui_renderer_frame(RTuiRenderer *renderer, bool begin);

enum RTuiError rtui_renderer_get_surface(RTuiRenderer *renderer, RTuiSurface **out_surface);

enum RTuiError rtui_renderer_shutdown(RTuiRenderer *renderer);

enum RTuiError rtui_surface_create(uint16_t width, uint16_t height, RTuiSurface **out_surface);

void rtui_surface_destroy(RTuiSurface *surface);

enum RTuiError rtui_surface_get_dimensions(const RTuiSurface *surface,
                                           struct RTuiDimensions *out_dimensions);

enum RTuiError rtui_surface_clear(RTuiSurface *surface, uint8_t r, uint8_t g, uint8_t b);

enum RTuiError rtui_surface_set_cell(RTuiSurface *surface,
                                     uint16_t x,
                                     uint16_t y,
                                     const struct RTuiCell *cell);

enum RTuiError rtui_surface_get_cell(const RTuiSurface *surface,
                                     uint16_t x,
                                     uint16_t y,
                                     struct RTuiCell *out_cell);

enum RTuiError rtui_surface_draw_text(RTuiSurface *surface,
                                      uint16_t x,
                                      uint16_t y,
                                      const char *text,
                                      const struct RTuiColor *fg,
                                      const struct RTuiColor *bg);

enum RTuiError rtui_surface_fill_rect(RTuiSurface *surface,
                                      const struct RTuiRect *rect,
                                      uint32_t ch,
                                      const struct RTuiColor *fg,
                                      const struct RTuiColor *bg);

enum RTuiError rtui_text_input_create(const char *placeholder,
                                      const char *initial_value,
                                      RTuiElement **out_element);

enum RTuiError rtui_checkbox_create(const char *label,
                                    bool initial_checked,
                                    RTuiElement **out_element);

enum RTuiError rtui_progress_bar_create(double min_value,
                                        double max_value,
                                        double current_value,
                                        const char *label,
                                        RTuiElement **out_element);

enum RTuiError rtui_button_create(const char *text, RTuiElement **out_element);

enum RTuiError rtui_text_element_create(const char *text, RTuiElement **out_element);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* REACTIVE_TUI_NATIVE_H */
