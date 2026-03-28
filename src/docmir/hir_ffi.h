#pragma once

#ifdef __cplusplus
extern "C" {
#endif

    #include <stdint.h>
    #include <stddef.h>

    /// Opaque handles
    typedef struct HLIR_Module HLIR_Module;
    typedef struct DOC_Module DOC_Module;

    /// Error code and message
    typedef struct {
        int32_t code;
        const char* message; // null-terminated, owned by callee
    } HLIR_Error;

    void hlir_init(void);
    void hlir_destroy(void);

    /// Lower HLIR → DOC using MLIR
    DOC_Module* hlir_lower_to_doc(const HLIR_Module* module, HLIR_Error* out_error);

    void doc_module_destroy(DOC_Module* module);

    /// Serialize DOC to a compact binary form
    /// (Rust-friendly, zero MLIR types leak through)
    uint8_t* doc_module_serialize(const DOC_Module* module, size_t* out_size, HLIR_Error* out_error);

    /// Free buffers allocated by the FFI layer
    void hlir_free_buffer(void* buffer);

#ifdef __cplusplus
}
#endif
