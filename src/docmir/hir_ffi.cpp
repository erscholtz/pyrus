#include <iostream>
#include <vector>

#include "hlir_ffi.h"

struct HLIR_Module {
    const void* rust_ptr; // Rust HLIR_Module
};

struct DOC_Module {
    std::vector<uint8_t> data;
};

void hlir_init(void) {
    std::cout << "HLIR initialized" << std::endl;
}

void hlir_destroy(void) {
    std::cout << "HLIR destroyed" << std::endl;
}

DOC_Module* hlir_lower_to_doc(const HLIR_Module* module, HLIR_Error* out_error) {
    if (!module) {
        if (out_error) {
            out_error->code = 1;
            out_error->message = "Input module is null";
        }
        return nullptr;
    }

    try {
        DOC_Module* doc = new DOC_Module();
        // Dummy transformation
        doc->data = {1, 2, 3, 4};
        if (out_error) {
            out_error->code = 0;
            out_error->message = nullptr;
        }
        return doc;
    } catch (const std::exception& e) {
        if (out_error) {
            out_error->code = 2;
            out_error->message = e.what();
        }
        return nullptr;
    }
}

void doc_module_destroy(DOC_Module* module) {
    delete module;
}

uint8_t* doc_module_serialize(const DOC_Module* module, size_t* out_size, HLIR_Error* out_error) {
    if (!module) {
        if (out_error) {
            out_error->code = 1;
            out_error->message = "DOC module is null";
        }
        return nullptr;
    }

    try { // TODO see if this can be done without try catch
        const std::vector<uint8_t>& data = module->data;
        uint8_t* buffer = new uint8_t[data.size()];
        std::memcpy(buffer, data.data(), data.size());
        if (out_size) *out_size = data.size();
        if (out_error) {
            out_error->code = 0;
            out_error->message = nullptr;
        }
        return buffer;
    } catch (const std::exception& e) {
        if (out_error) {
            out_error->code = 2;
            out_error->message = e.what();
        }
        return nullptr;
    }
}

void hlir_free_buffer(void* buffer) {
    delete[] static_cast<uint8_t*>(buffer);
}
