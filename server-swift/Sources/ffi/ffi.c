#include "ffi.h"

#ifdef _WIN32
#include <windows.h>
#include <stdlib.h>
#include <wchar.h>
#include <string.h>

// The official llama archive ships ggml.dll without a ggml import library.
// Load its backend-discovery API from the selected backend's absolute path.
int azookey_load_backends(void) {
    const wchar_t *directory = _wgetenv(L"AZOOKEY_BACKEND_PATH");
    if (!directory || !*directory) return 0;
    const wchar_t suffix[] = L"\\ggml.dll";
    size_t directory_length = wcslen(directory);
    wchar_t *path = malloc(directory_length * sizeof(wchar_t) + sizeof(suffix));
    if (!path) return 0;
    memcpy(path, directory, directory_length * sizeof(wchar_t));
    memcpy(path + directory_length, suffix, sizeof(suffix));
    HMODULE library = LoadLibraryW(path);
    free(path);
    if (!library) return 0;
    typedef void (__cdecl *load_backends_fn)(const char *);
    load_backends_fn load = (load_backends_fn)GetProcAddress(library, "ggml_backend_load_all_from_path");
    if (!load) { FreeLibrary(library); return 0; }
    int length = WideCharToMultiByte(CP_UTF8, 0, directory, -1, NULL, 0, NULL, NULL);
    char *utf8 = length > 0 ? malloc((size_t)length) : NULL;
    if (!utf8) { FreeLibrary(library); return 0; }
    if (!WideCharToMultiByte(CP_UTF8, 0, directory, -1, utf8, length, NULL, NULL)) {
        free(utf8);
        FreeLibrary(library);
        return 0;
    }
    load(utf8);
    free(utf8);
    // Keep ggml loaded for the lifetime of the converter's model and devices.
    return 1;
}
#else
int azookey_load_backends(void) { return 0; }
#endif
