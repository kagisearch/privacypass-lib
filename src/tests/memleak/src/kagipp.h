#ifndef KAGIPP
#define KAGIPP

// File: kagipp.h

#include <stdint.h>

char * gen_keys();

// // Some example C functions that returns a string that has been
// // allocated on the heap.  The caller must call free on s to
// // prevent a memory leak.  Look at implementation of each to
// // see differences.
// int get_some_cstr( char **s );
// int get_some_cstr_2( char **s );

// // Opaque type for some Error
// typedef struct _custom_error Error;

// // Create function which takes a pointer to a pointer for returning
// // the newly allocated type and can also return error codes.
// int32_t error_create_with_result(Error **o);


// // Free function which takes a pointer to a pointer for freeing
// // the memory, returns error code based on ERRNO.
// int32_t error_free_with_result(Error **o);


// // An alternative simpiler function for allocating a type which
// // can only communicate success or fail based on if the returned
// // value is non-null.
// Error* error_new(void);

// // Alternative free function which simply takes a pointer to type to
// // de-allocate the memory.  See implementation on how it varies.
// void error_free(Error *o);


// // Common "getter" C functions which operate on the opaque type.
// const char* error_msg_get(const Error *o);
// int error_code_get(const Error *o);

#endif