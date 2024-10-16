#include <stdio.h>
#include <stdint.h>
#include <complex.h>
#include <stdbool.h>

int main() {
    return 0;
}


/// [settings]
/// run = true
/// stdin = true
///
/// [test]
/// input = [
///     5, 
///     5,
///     3.5,
///     {define = "NULL"}
/// ]
bool cesty_testy(complex double i, complex double x, float y, const char * name ){

    if(name == NULL) {
        return false;
    }

    int result = i+1;

    if(result >= 0) {
        return true;
    }

    return false;

}

int rando_funco(int i, int x, float y, const char * name){
    return i+1;
}

