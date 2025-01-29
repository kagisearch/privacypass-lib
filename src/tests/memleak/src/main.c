#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include "kagipp.h"

int main(void) {

  for (int i = 0; i < 10; i++) {
    char *keypair = gen_keys();
    if (keypair) {
        printf("keypair = %s\n", keypair);
    } else {
        printf("Error, keypair is null :-/\n");
        return 1;
    }
    free(keypair);
  }

  return 0;
}
