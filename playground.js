/* START builtins */

function _printf(msg) {
  process.stdout.write(msg);
}

/* END builtins */
function print(msg){
_printf(msg);
}
function println(msg){
print(msg + "\n");
}
function len(arr){
var c = 0;
while (arr[c]) {
c = c + 1;
}
;
return c;
}
function rev(arr){
var l = len(arr);
var new_arr = [];
var i = 0;
var j = l;
while (i < l) {
new_arr[i] = arr[j];
i = i - 1;
j = j - 1;
}
;
return new_arr;
}
function main(){
var arr = [1, 4, 2, 5, 3];
}
main();