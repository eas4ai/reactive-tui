#include "reactive_tui/native.h"
#include <assert.h>
#include <pthread.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define OK(call) assert((call) == 0)

typedef struct {
    RTuiForeignComponent *component;
    RTuiTextEditor *editor;
    RTuiDialogEngine *dialogs;
    int rendered, events, disposed;
    bool fail_render, fail_event, missing_element;
    char result[256];
} Context;

static RTuiElement *text(const char *value) {
    RTuiElement *element = NULL;
    OK(rtui_element_create_text(value, &element));
    return element;
}

static void style(RTuiElement *element, const char *css) {
    RTuiNativeStyle *value = NULL;
    OK(rtui_native_style_create(css, &value));
    OK(rtui_native_style_apply(value, element));
    rtui_native_style_destroy(value);
}

static RTuiElement *column(void) {
    RTuiElement *element = NULL;
    OK(rtui_element_create_layout(R_TUI_LAYOUT_TYPE_FLEX, &element));
    style(element, "display:flex;flex-direction:column;width:100%;height:100%");
    return element;
}

static int32_t foreign_render(const char *props, const char *state, void *data, RTuiElement **out) {
    Context *ctx = data;
    ctx->rendered++;
    /* Native code must not retain its state lock while invoking this callback. */
    char *copy = NULL;
    OK(rtui_foreign_component_get_state(ctx->component, &copy));
    assert(strcmp(copy, state) == 0);
    rtui_string_free(copy);
    RTuiElement *recursive = NULL;
    assert(rtui_foreign_component_render(ctx->component, &recursive) == -10);
    assert(recursive == NULL);
    assert(rtui_foreign_component_destroy(ctx->component) == -10);
    if (ctx->missing_element) return 0;
    if (ctx->fail_render) {
        *out = text("returned allocation on failure");
        return -1;
    }
    if (ctx->dialogs) {
        char *event = NULL;
        while (rtui_dialog_engine_take_event(ctx->dialogs, &event) == 0 && event) {
            if (strstr(event, "\"kind\":\"closed\"")) {
                assert(strlen(event) < sizeof(ctx->result));
                strcpy(ctx->result, event);
            }
            rtui_string_free(event);
            event = NULL;
        }
    }
    char status[512];
    const char *completion = strstr(ctx->result, "\"kind\":\"confirmed\"") ? "confirmed"
        : strstr(ctx->result, "\"kind\":\"cancelled\"") ? "cancelled"
        : strstr(ctx->result, "\"kind\":\"selected\"") ? "selected" : "none";
    snprintf(status, sizeof(status), "native %s count%s %s completion=%s", props, state,
             strstr(ctx->result, "Ada") ? "result=Ada" : "ready", completion);
    RTuiElement *root = column();
    RTuiElement *label = text(status);
    OK(rtui_element_set_key(label, "status"));
    OK(rtui_element_set_focus(label, true, true));
    OK(rtui_element_add_child(root, label));
    RTuiElement *row = column();
    style(row, "display:flex;flex-direction:row;gap:3;height:1;flex-shrink:0");
    OK(rtui_element_add_child(row, text("Left")));
    OK(rtui_element_add_child(row, text("Right")));
    OK(rtui_element_add_child(root, row));
    if (ctx->editor) {
        RTuiElement *editor = NULL;
        OK(rtui_text_editor_element(ctx->editor, &editor));
        OK(rtui_element_add_child(root, editor));
    }
    if (ctx->dialogs) {
        RTuiElement *dialogs = NULL;
        OK(rtui_dialog_engine_element(ctx->dialogs, &dialogs));
        OK(rtui_element_add_child(root, dialogs));
    }
    *out = root;
    return 0;
}

static int32_t event(const char *input, const char *props, const char *state,
                     void *data, bool *handled) {
    Context *ctx = data;
    ctx->events++;
    assert(props && state);
    bool recursive = false;
    assert(rtui_foreign_component_dispatch(ctx->component, input, &recursive) == -10);
    assert(rtui_foreign_component_destroy(ctx->component) == -10);
    if (ctx->fail_event) return -12;
    *handled = true;
    if (strstr(input, "\"key\":\"x\"") || strstr(input, "\"kind\":\"down\"")) {
#ifdef API_NATIVE_NEGATIVE
        return 0; /* Deliberately broken: acknowledge input without updating state. */
#endif
        char next[32];
        snprintf(next, sizeof(next), "%d", atoi(state) + 1);
        OK(rtui_foreign_component_set_state(ctx->component, next));
    } else if (strstr(input, "\"key\":\"p\"")) {
        OK(rtui_foreign_component_set_props(ctx->component, "\"omega\""));
    } else if (strstr(input, "\"key\":\"e\"")) {
        OK(rtui_text_editor_insert_text(ctx->editor, "界é"));
        OK(rtui_foreign_component_set_state(ctx->component, state));
    } else if (strstr(input, "\"key\":\"d\"")) {
        uint32_t id = 0;
        OK(rtui_dialog_engine_open(ctx->dialogs,
            "{\"kind\":\"input\",\"title\":\"Name\",\"prompt\":\"Enter name\"}", &id));
        assert(id != 0);
    } else {
        *handled = false;
    }
    return 0;
}

static void dispose(void *data) { ((Context *)data)->disposed++; }

static void create(Context *ctx) {
    memset(ctx, 0, sizeof(*ctx));
    OK(rtui_foreign_component_create("\"alpha\"", "0", foreign_render, event, dispose, ctx, &ctx->component));
}

static void *wrong_thread(void *data) {
    Context *ctx = data;
    RTuiElement *out = NULL;
    assert(rtui_foreign_component_render(ctx->component, &out) == -10);
    assert(rtui_foreign_component_destroy(ctx->component) == -10);
    return NULL;
}

static void units(void) {
    Context a, b;
    create(&a); create(&b);
    pthread_t thread;
    assert(pthread_create(&thread, NULL, wrong_thread, &a) == 0);
    assert(pthread_join(thread, NULL) == 0);
    assert(a.rendered == 0 && a.disposed == 0);
    RTuiElement *snapshot = NULL;
    OK(rtui_foreign_component_element(a.component, &snapshot));
    RTuiElement *paint = NULL;
    OK(rtui_foreign_component_render(a.component, &paint));
    rtui_element_destroy(paint);
    bool handled = false;
    OK(rtui_foreign_component_dispatch(a.component, "{\"type\":\"key\",\"key\":\"x\"}", &handled));
    assert(handled);
    char *value = NULL;
    OK(rtui_foreign_component_get_state(a.component, &value));
    assert(strcmp(value, "1") == 0); rtui_string_free(value);
    OK(rtui_foreign_component_get_state(b.component, &value));
    assert(strcmp(value, "0") == 0); rtui_string_free(value);
    assert(rtui_foreign_component_set_state(a.component, "{broken") == -1);
    a.missing_element = true;
    paint = NULL;
    assert(rtui_foreign_component_render(a.component, &paint) == -10);
    assert(paint == NULL);
    a.missing_element = false;
    a.fail_render = true;
    paint = NULL;
    assert(rtui_foreign_component_render(a.component, &paint) == -1);
    assert(paint == NULL);
    a.fail_event = true;
    assert(rtui_foreign_component_dispatch(a.component, "{}", &handled) == -12);
    /* Clear callback failure before testing disposal, so it cannot mask a
       broken disposed-controller guard. Direct render is an explicit retry. */
    a.fail_render = false; a.fail_event = false;
    OK(rtui_foreign_component_render(a.component, &paint));
    rtui_element_destroy(paint);
    OK(rtui_foreign_component_destroy(a.component));
    assert(a.disposed == 1);
    /* A retained Element cannot invoke callbacks after explicit disposal. */
    int before = a.rendered;
    RTuiAppBuilder *builder = NULL;
    RTuiApp *app = NULL;
    OK(rtui_app_builder_create(&builder));
    OK(rtui_app_builder_backend_debug(builder, 16, 2));
    OK(rtui_app_builder_root_element(builder, snapshot));
    OK(rtui_app_builder_build(builder, &app));
    assert(rtui_app_run(app) != 0);
    assert(a.rendered == before);
    assert(a.disposed == 1);
    OK(rtui_foreign_component_destroy(b.component));

    RTuiTextEditor *editor = NULL;
    OK(rtui_text_editor_create(&editor));
    OK(rtui_text_editor_insert_text(editor, "A界é👩🏽‍💻"));
    OK(rtui_text_editor_delete(editor, true));
    OK(rtui_text_editor_get_content_owned(editor, &value));
    assert(strcmp(value, "A界é") == 0); rtui_string_free(value);
    OK(rtui_text_editor_move(editor, 0, true)); /* Left selects one grapheme. */
    OK(rtui_text_editor_insert_text(editor, "!"));
    OK(rtui_text_editor_get_content_owned(editor, &value));
    assert(strcmp(value, "A界!") == 0); rtui_string_free(value);
    assert(rtui_text_editor_move(editor, 999, false) == -1);
    assert(rtui_text_editor_set_size(editor, UINT32_MAX, 2) == -1);
    rtui_text_editor_destroy(editor);
    RTuiNativeStyle *css = NULL;
    assert(rtui_native_style_create("width:nan", &css) == -1 && css == NULL);
    assert(rtui_native_style_create("made-up:value", &css) == -1 && css == NULL);

    RTuiDialogEngine *engine = NULL;
    OK(rtui_dialog_engine_create(&engine));
    const char *kinds[] = {"confirmation", "input", "toast", "progress", "autocomplete", "wizard"};
    for (size_t i = 0; i < sizeof(kinds) / sizeof(*kinds); i++) {
        char options[256];
        snprintf(options, sizeof(options), "{\"kind\":\"%s\",\"title\":\"Test\"}", kinds[i]);
        uint32_t id = 0;
        OK(rtui_dialog_engine_open(engine, options, &id));
        OK(rtui_dialog_engine_take_event(engine, &value));
        assert(value && strstr(value, "\"kind\":\"opened\"")); rtui_string_free(value);
        if (strcmp(kinds[i], "progress") == 0)
            OK(rtui_dialog_engine_update(engine, id, "{\"progress\":0.5}"));
        if (strcmp(kinds[i], "input") == 0)
            OK(rtui_dialog_engine_update(engine, id, "{\"input\":\"edited\"}"));
        OK(rtui_dialog_engine_close(engine, id, "{\"kind\":\"confirmed\",\"data\":\"saved\"}"));
        assert(rtui_dialog_engine_close(engine, id, "{\"kind\":\"cancelled\"}") == -9);
        OK(rtui_dialog_engine_take_event(engine, &value));
        assert(value && strstr(value, "\"kind\":\"closed\"") && strstr(value, "saved"));
        rtui_string_free(value);
    }
    value = (char *)(uintptr_t)1;
    OK(rtui_dialog_engine_take_event(engine, &value)); assert(value == NULL);
    uint32_t invalid = 0;
    assert(rtui_dialog_engine_open(engine, "{\"kind\":\"bogus\"}", &invalid) == -1);
    assert(rtui_dialog_engine_open(engine, "{\"kind\":\"input\",\"made_up\":true}", &invalid) == -1);
    assert(rtui_dialog_engine_update(engine, UINT32_MAX, "{\"input\":\"x\"}") == -9);
    rtui_dialog_engine_destroy(engine);
    puts("NATIVE_COMPONENT_UNITS_PASS");
}

static RTuiElement *root(void *data) {
    RTuiElement *element = NULL;
    OK(rtui_foreign_component_element(((Context *)data)->component, &element));
    return element;
}

int main(int argc, char **argv) {
    OK(rtui_init());
    if (argc == 1) { units(); rtui_cleanup(); return 0; }
    Context ctx; create(&ctx);
    OK(rtui_text_editor_create(&ctx.editor));
    OK(rtui_text_editor_set_size(ctx.editor, 24, 2));
    OK(rtui_text_editor_set_show_line_numbers(ctx.editor, false));
    OK(rtui_text_editor_insert_text(ctx.editor, "Edit:"));
    OK(rtui_dialog_engine_create(&ctx.dialogs));
    if (strcmp(argv[1], "dialog") == 0) {
        assert(argc == 3);
        uint32_t id = 0;
        OK(rtui_dialog_engine_open(ctx.dialogs, argv[2], &id));
        if (strstr(argv[2], "\"kind\":\"progress\""))
            OK(rtui_dialog_engine_update(ctx.dialogs, id, "{\"progress\":0.75}"));
    }
    ctx.fail_render = strcmp(argv[1], "error") == 0;
    ctx.fail_event = strcmp(argv[1], "event-error") == 0;
    RTuiAppBuilder *builder = NULL;
    RTuiApp *app = NULL;
    OK(rtui_app_builder_create(&builder));
    OK(rtui_app_builder_backend_suprtui(builder));
    OK(rtui_app_builder_root_component(builder, root, &ctx));
    OK(rtui_app_builder_build(builder, &app));
    int result = rtui_app_run(app);
    if (ctx.fail_render || ctx.fail_event) {
        assert(result != 0);
        int32_t cause = 0;
        OK(rtui_foreign_component_last_error(ctx.component, &cause));
        assert(cause == (ctx.fail_render ? -1 : -12));
    } else assert(result == 0);
    OK(rtui_foreign_component_destroy(ctx.component));
    assert(ctx.disposed == 1 && ctx.rendered > 0);
    rtui_dialog_engine_destroy(ctx.dialogs);
    rtui_text_editor_destroy(ctx.editor);
    rtui_cleanup();
    puts("ENTRY_POINT_CLEAN_EXIT");
    return 0;
}
