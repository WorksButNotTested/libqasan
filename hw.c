#include <stdio.h>

extern void* gasan_allocate(size_t size, size_t align);
extern void gasan_deallocate(void* p);

int main() {
  printf("Hello, World!\n");
  void * p = gasan_allocate(8, 0);
  printf("p: %p\n", p);
  gasan_deallocate(p);
  return 0;
}