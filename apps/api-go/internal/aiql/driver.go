package aiql

/*
#cgo LDFLAGS: -L../../../../target/release -laiql_ffi -ldl -lpthread -lm
#include <stdlib.h>

extern char* aiql_crawl_postgres(const char* db_url);
extern void aiql_free_string(char* s);
*/
import "C"
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
