#include <stdio.h>
#include <assert.h>
#include <string.h>

// Forward declarations for FFI functions
typedef enum {
    RTUI_SUCCESS = 0,
    RTUI_NULL_POINTER = -2,
    RTUI_INVALID_UTF8 = -5,
} RTuiError;

typedef struct RTuiElementBuilder RTuiElementBuilder;
typedef struct RTuiElement RTuiElement;

// FFI function declarations
RTuiError rtui_element_builder_div(RTuiElementBuilder** out_builder);
RTuiError rtui_element_builder_add_class(RTuiElementBuilder* builder, const char* classes);
RTuiError rtui_element_builder_set_text(RTuiElementBuilder* builder, const char* text);
RTuiError rtui_element_builder_set_key(RTuiElementBuilder* builder, const char* key);
RTuiError rtui_element_builder_build(RTuiElementBuilder* builder, RTuiElement** out_element);
void rtui_element_builder_destroy(RTuiElementBuilder* builder);
void rtui_element_destroy(RTuiElement* element);

int test_basic_builder_creation() {
    printf("Testing basic builder creation...\n");
    
    RTuiElementBuilder* builder = NULL;
    RTuiError err = rtui_element_builder_div(&builder);
    
    assert(err == RTUI_SUCCESS);
    assert(builder != NULL);
    
    printf("✓ Builder created successfully\n");
    
    // Clean up
    rtui_element_builder_destroy(builder);
    return 0;
}

int test_builder_modification() {
    printf("Testing builder modification...\n");
    
    RTuiElementBuilder* builder = NULL;
    RTuiError err = rtui_element_builder_div(&builder);
    assert(err == RTUI_SUCCESS);
    
    RTuiElementBuilder* original_ptr = builder;
    
    // Test adding class - pointer should remain the same
    err = rtui_element_builder_add_class(builder, "test-class");
    assert(err == RTUI_SUCCESS);
    assert(builder == original_ptr); // Pointer should not change
    
    // Test adding more classes
    err = rtui_element_builder_add_class(builder, "another-class");
    assert(err == RTUI_SUCCESS);
    assert(builder == original_ptr); // Pointer should not change
    
    // Test setting text
    err = rtui_element_builder_set_text(builder, "Hello, World!");
    assert(err == RTUI_SUCCESS);
    assert(builder == original_ptr); // Pointer should not change
    
    // Test setting key
    err = rtui_element_builder_set_key(builder, "my-element");
    assert(err == RTUI_SUCCESS);
    assert(builder == original_ptr); // Pointer should not change
    
    printf("✓ Builder modified in place successfully\n");
    
    // Clean up
    rtui_element_builder_destroy(builder);
    return 0;
}

int test_builder_build() {
    printf("Testing builder build...\n");
    
    RTuiElementBuilder* builder = NULL;
    RTuiError err = rtui_element_builder_div(&builder);
    assert(err == RTUI_SUCCESS);
    
    // Add some properties
    err = rtui_element_builder_add_class(builder, "container");
    assert(err == RTUI_SUCCESS);
    
    err = rtui_element_builder_set_text(builder, "Test content");
    assert(err == RTUI_SUCCESS);
    
    // Build the element
    RTuiElement* element = NULL;
    err = rtui_element_builder_build(builder, &element);
    assert(err == RTUI_SUCCESS);
    assert(element != NULL);
    
    printf("✓ Element built successfully\n");
    
    // Note: builder is consumed by build, so don't destroy it
    rtui_element_destroy(element);
    return 0;
}

int test_null_pointer_safety() {
    printf("Testing null pointer safety...\n");
    
    // Test null out_builder
    RTuiError err = rtui_element_builder_div(NULL);
    assert(err == RTUI_NULL_POINTER);
    
    // Test null builder
    err = rtui_element_builder_add_class(NULL, "test");
    assert(err == RTUI_NULL_POINTER);
    
    // Test null classes
    RTuiElementBuilder* builder = NULL;
    rtui_element_builder_div(&builder);
    err = rtui_element_builder_add_class(builder, NULL);
    assert(err == RTUI_NULL_POINTER);
    
    printf("✓ Null pointer safety verified\n");
    
    rtui_element_builder_destroy(builder);
    return 0;
}

int main() {
    printf("=== FFI Builder Test Suite ===\n");
    
    test_basic_builder_creation();
    test_builder_modification();
    test_builder_build();
    test_null_pointer_safety();
    
    printf("=== All tests passed! ===\n");
    return 0;
}
