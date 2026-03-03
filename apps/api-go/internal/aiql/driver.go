package aiql

/*
#cgo LDFLAGS: -L../../../../target/release -laiql_ffi -ldl -lpthread -lm
#include <stdlib.h>

extern char* aiql_crawl_postgres(const char* db_url);
extern char* aiql_translate(const char* prompt, const char* schema_json);
extern void aiql_free_string(char* s);
*/
import "C"
// ...
func Translate(prompt, schemaJSON string) (string, error) {
	cPrompt := C.CString(prompt)
	defer C.free(unsafe.Pointer(cPrompt))
	cSchemaJSON := C.CString(schemaJSON)
	defer C.free(unsafe.Pointer(cSchemaJSON))

	cResult := C.aiql_translate(cPrompt, cSchemaJSON)
	if cResult == nil {
		return "", fmt.Errorf("failed to translate")
	}
	defer C.aiql_free_string(cResult)

	return C.GoString(cResult), nil
}
import (
	"fmt"
	"unsafe"
)

func CrawlPostgres(dbURL string) (string, error) {
	cDbURL := C.CString(dbURL)
	defer C.free(unsafe.Pointer(cDbURL))

	cResult := C.aiql_crawl_postgres(cDbURL)
	if cResult == nil {
		return "", fmt.Errorf("failed to crawl postgres")
	}
	defer C.aiql_free_string(cResult)

	return C.GoString(cResult), nil
}
