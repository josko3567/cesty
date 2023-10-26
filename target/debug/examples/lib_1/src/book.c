// #include <stdio.h>
#include <stdint.h>
#include "../lib.h"
#define hello(x) ({ \
	int i = 0;     \
	i; \
})
/**
<!-- #!cesty;
---  
info:
	standalone: true
warn: true
	run: true
test:
	- name: def
	  code: >
		struct innerbook = my_book();
		if(innerbook.author == BOOK_AUTHOR
        && innerbook.name == BOOK_NAME
        && innerbook.sold_amount == BOOK_SALES ) {
		
			return true;

		}
		return false;
	  expect: true
execute:
	- make -C ".."

compiler:
	global:
		name: override
	name: clang	
	flags: -std=c11
...--->

returns a copy of the default book.
# Examples
```C
assert(95923 == my_book().sold_amount);
```
*/
__attribute__((deprecated))
struct innerbook my_book()
{
	int foobar(int i) {
		return i+2;
	}

	int i = foobar(foo(bar(2)));
	return (struct innerbook){
		.author = BOOK_AUTHOR,
		.name = BOOK_NAME,
		.sold_amount = BOOK_SALES+i
	};

}

int fooishlybarful(int i) {

	return 100+i;

}