package aiql

/*
#cgo LDFLAGS: -L../../../../target/release -laiql_ffi -ldl -lpthread -lm
#include <stdlib.h>

extern char* aiql_crawl_postgres(const char* db_url);
extern char* aiql_translate(const char* prompt, const char* schema_json);
extern char* aiql_ask(const char* prompt, const char* db_url, const char* schema_json);
extern char* aiql_generate_mock_data(const char* prompt, const char* schema_json);
extern void aiql_free_string(char* s);
*/
import "C"
// ...
func GenerateMockData(prompt, schemaJSON string) (string, error) {
	cPrompt := C.CString(prompt)
	defer C.free(unsafe.Pointer(cPrompt))
	cSchemaJSON := C.CString(schemaJSON)
	defer C.free(unsafe.Pointer(cSchemaJSON))

	cResult := C.aiql_generate_mock_data(cPrompt, cSchemaJSON)
	if cResult == nil {
		return "", fmt.Errorf("failed to generate mock data")
	}
	defer C.aiql_free_string(cResult)

	return C.GoString(cResult), nil
}
// ...
func Ask(prompt, dbURL, schemaJSON string) (string, error) {
	cPrompt := C.CString(prompt)
	defer C.free(unsafe.Pointer(cPrompt))
	cDbURL := C.CString(dbURL)
	defer C.free(unsafe.Pointer(cDbURL))
	cSchemaJSON := C.CString(schemaJSON)
	defer C.free(unsafe.Pointer(cSchemaJSON))

	cResult := C.aiql_ask(cPrompt, cDbURL, cSchemaJSON)
	if cResult == nil {
		return "", fmt.Errorf("failed to execute smart ask")
	}
	defer C.aiql_free_string(cResult)

	return C.GoString(cResult), nil
}
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
