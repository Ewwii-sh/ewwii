---
title: global
slug: /global
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';



# Global Builtin Rhai Functions

These functions are built-in and available globally, meaning they can be used directly without any import.

For example, to get the value of PI, you can simply write:

```javascript
let x = PI();
```

This section covers all the core functions provided by Rhai that are ready to use out of the box.
    




## <code>op</code> !&#x3D; {#op-!&#x3D;}

```js
op u8 != u8 -> bool
op f32 != f32 -> bool
op u128 != u128 -> bool
op Instant != Instant -> bool
op i32 != i32 -> bool
op u32 != u32 -> bool
op i16 != i16 -> bool
op i128 != i128 -> bool
op Array != Array -> bool
op u64 != u64 -> bool
op u16 != u16 -> bool
op i8 != i8 -> bool
op int != f32 -> bool
op f32 != int -> bool
op Map != Map -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if two timestamps are not equal.
    </TabItem>
</Tabs>

## <code>fn</code> + {#fn-+}

```js
fn +(x: f32) -> f32
fn +(x: int) -> int
fn +(x: i128) -> i128
fn +(x: float) -> float
fn +(x: i16) -> i16
fn +(x: i32) -> i32
fn +(x: i8) -> i8
fn +(x: i8, y: i8) -> i8
fn +(item: ?, string: String) -> String
fn +(string: String, utf8: Blob) -> String
fn +(array1: Array, array2: Array) -> Array
fn +(string: String, character: char) -> String
fn +(x: u64, y: u64) -> u64
fn +(x: u16, y: u16) -> u16
fn +(x: f32, y: int) -> f32
fn +(map1: Map, map2: Map) -> Map
fn +(utf8: Blob, string: String) -> String
fn +(x: int, y: f32) -> f32
fn +(item: ?, string: String) -> String
fn +(timestamp: Instant, seconds: float) -> Instant
fn +(x: u8, y: u8) -> u8
fn +(x: i16, y: i16) -> i16
fn +(string: String, mut item: ?) -> String
fn +(string1: String, string2: String) -> String
fn +(string: String, item: ?) -> String
fn +(x: i128, y: i128) -> i128
fn +(character: char, string: String) -> String
fn +(x: f32, y: f32) -> f32
fn +(x: u128, y: u128) -> u128
fn +(x: u32, y: u32) -> u32
fn +(x: i32, y: i32) -> i32
fn +(timestamp: Instant, seconds: int) -> Instant
```

<Tabs>
    <TabItem value="Description" default>

        Combine two arrays into a new array and return it.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = [1, 2, 3];
        let y = [true, 'x'];
        
        print(x + y);   // prints "[1, 2, 3, true, 'x']"
        
        print(x);       // prints "[1, 2, 3"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> +&#x3D; {#fn-+&#x3D;}

```js
fn +=(string: String, character: char)
fn +=(string: String, utf8: Blob)
fn +=(map: Map, map2: Map)
fn +=(string: String, mut item: ?)
fn +=(timestamp: Instant, seconds: float)
fn +=(string: String, item: ?)
fn +=(timestamp: Instant, seconds: int)
fn +=(string1: String, string2: String)
```

<Tabs>
    <TabItem value="Description" default>

        Add all property values of another object map into the object map.
        Existing property values of the same names are replaced.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let m = #{a:1, b:2, c:3};
        let n = #{a: 42, d:0};
        
        m.mixin(n);
        
        print(m);       // prints "#{a:42, b:2, c:3, d:0}"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> - {#fn--}

```js
fn -(x: i8) -> i8
fn -(x: float) -> float
fn -(x: i16) -> i16
fn -(x: i32) -> i32
fn -(x: i128) -> i128
fn -(x: int) -> int
fn -(x: f32) -> f32
fn -(x: u8, y: u8) -> u8
fn -(timestamp: Instant, seconds: float) -> Instant
fn -(x: i128, y: i128) -> i128
fn -(x: i16, y: i16) -> i16
fn -(x: f32, y: f32) -> f32
fn -(x: i32, y: i32) -> i32
fn -(timestamp: Instant, seconds: int) -> Instant
fn -(x: u32, y: u32) -> u32
fn -(x: u128, y: u128) -> u128
fn -(timestamp1: Instant, timestamp2: Instant) -> ?
fn -(x: i8, y: i8) -> i8
fn -(x: u16, y: u16) -> u16
fn -(x: u64, y: u64) -> u64
fn -(x: f32, y: int) -> f32
fn -(x: int, y: f32) -> f32
```

<Tabs>
    <TabItem value="Description" default>

        Subtract the specified number of `seconds` from the timestamp and return it as a new timestamp.
    </TabItem>
</Tabs>

## <code>fn</code> -&#x3D; {#fn--&#x3D;}

```js
fn -=(timestamp: Instant, seconds: int)
fn -=(timestamp: Instant, seconds: float)
```

<Tabs>
    <TabItem value="Description" default>

        Subtract the specified number of `seconds` from the timestamp.
    </TabItem>
</Tabs>

## <code>op</code> &lt; {#op-&lt;}

```js
op int < f32 -> bool
op f32 < int -> bool
op u16 < u16 -> bool
op u64 < u64 -> bool
op i8 < i8 -> bool
op f32 < f32 -> bool
op i32 < i32 -> bool
op u32 < u32 -> bool
op u128 < u128 -> bool
op Instant < Instant -> bool
op i128 < i128 -> bool
op i16 < i16 -> bool
op u8 < u8 -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the first timestamp is earlier than the second.
    </TabItem>
</Tabs>

## <code>op</code> &lt;&#x3D; {#op-&lt;&#x3D;}

```js
op f32 <= int -> bool
op int <= f32 -> bool
op i8 <= i8 -> bool
op u16 <= u16 -> bool
op u64 <= u64 -> bool
op i128 <= i128 -> bool
op i16 <= i16 -> bool
op f32 <= f32 -> bool
op u32 <= u32 -> bool
op i32 <= i32 -> bool
op u128 <= u128 -> bool
op Instant <= Instant -> bool
op u8 <= u8 -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the first timestamp is earlier than or equals to the second.
    </TabItem>
</Tabs>

## <code>op</code> &#x3D;&#x3D; {#op-&#x3D;&#x3D;}

```js
op u8 == u8 -> bool
op f32 == f32 -> bool
op Instant == Instant -> bool
op u128 == u128 -> bool
op u32 == u32 -> bool
op i32 == i32 -> bool
op i16 == i16 -> bool
op i128 == i128 -> bool
op Array == Array -> bool
op u64 == u64 -> bool
op u16 == u16 -> bool
op i8 == i8 -> bool
op int == f32 -> bool
op f32 == int -> bool
op Map == Map -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if two timestamps are equal.
    </TabItem>
</Tabs>

## <code>op</code> &gt; {#op-&gt;}

```js
op i8 > i8 -> bool
op u64 > u64 -> bool
op u16 > u16 -> bool
op f32 > int -> bool
op int > f32 -> bool
op u8 > u8 -> bool
op i128 > i128 -> bool
op i16 > i16 -> bool
op u32 > u32 -> bool
op i32 > i32 -> bool
op Instant > Instant -> bool
op u128 > u128 -> bool
op f32 > f32 -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the first timestamp is later than the second.
    </TabItem>
</Tabs>

## <code>op</code> &gt;&#x3D; {#op-&gt;&#x3D;}

```js
op u8 >= u8 -> bool
op f32 >= f32 -> bool
op i32 >= i32 -> bool
op u32 >= u32 -> bool
op u128 >= u128 -> bool
op Instant >= Instant -> bool
op i128 >= i128 -> bool
op i16 >= i16 -> bool
op u16 >= u16 -> bool
op u64 >= u64 -> bool
op i8 >= i8 -> bool
op int >= f32 -> bool
op f32 >= int -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the first timestamp is later than or equals to the second.
    </TabItem>
</Tabs>

## <code>get/set</code> ?.tag {#getset-?.tag}

```js
get ?.tag -> int
set ?.tag = int
```

<Tabs>
    <TabItem value="Description" default>

        Return the _tag_ of a `Dynamic` value.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = "hello, world!";
        
        x.tag = 42;
        
        print(x.tag);           // prints 42
        ```
    </TabItem>
</Tabs>

## <code>get/set</code> Array.is_empty {#getset-Array.is_empty}

```js
get Array.is_empty -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the array is empty.
    </TabItem>
</Tabs>

## <code>get/set</code> Array.len {#getset-Array.len}

```js
get Array.len -> int
```

<Tabs>
    <TabItem value="Description" default>

        Number of elements in the array.
    </TabItem>
</Tabs>

## <code>get/set</code> Blob.is_empty {#getset-Blob.is_empty}

```js
get Blob.is_empty -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the BLOB is empty.
    </TabItem>
</Tabs>

## <code>get/set</code> Blob.len {#getset-Blob.len}

```js
get Blob.len -> int
```

<Tabs>
    <TabItem value="Description" default>

        Return the length of the BLOB.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let b = blob(10, 0x42);
        
        print(b);           // prints "[4242424242424242 4242]"
        
        print(b.len());     // prints 10
        ```
    </TabItem>
</Tabs>

## <code>fn</code> E {#fn-E}

```js
fn E() -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the natural number _e_.
    </TabItem>
</Tabs>

## <code>get/set</code> FnPtr.is_anonymous {#getset-FnPtr.is_anonymous}

```js
get FnPtr.is_anonymous -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the function is an anonymous function.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let f = |x| x * 2;
        
        print(f.is_anonymous);      // prints true
        ```
    </TabItem>
</Tabs>

## <code>get/set</code> FnPtr.name {#getset-FnPtr.name}

```js
get FnPtr.name -> String
```

<Tabs>
    <TabItem value="Description" default>

        Return the name of the function.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        fn double(x) { x * 2 }
        
        let f = Fn("double");
        
        print(f.name);      // prints "double"
        ```
    </TabItem>
</Tabs>

## <code>get/set</code> Instant.elapsed {#getset-Instant.elapsed}

```js
get Instant.elapsed -> ?
```

<Tabs>
    <TabItem value="Description" default>

        Return the number of seconds between the current system time and the timestamp.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let now = timestamp();
        
        sleep(10.0);            // sleep for 10 seconds
        
        print(now.elapsed);     // prints 10.???
        ```
    </TabItem>
</Tabs>

## <code>fn</code> PI {#fn-PI}

```js
fn PI() -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the number π.
    </TabItem>
</Tabs>

## <code>get/set</code> Range&lt;int&gt;.end {#getset-Range&lt;int&gt;.end}

```js
get Range<int>.end -> int
```

<Tabs>
    <TabItem value="Description" default>

        Return the end of the exclusive range.
    </TabItem>
</Tabs>

## <code>get/set</code> Range&lt;int&gt;.is_empty {#getset-Range&lt;int&gt;.is_empty}

```js
get Range<int>.is_empty -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the range contains no items.
    </TabItem>
</Tabs>

## <code>get/set</code> Range&lt;int&gt;.is_exclusive {#getset-Range&lt;int&gt;.is_exclusive}

```js
get Range<int>.is_exclusive -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the range is exclusive.
    </TabItem>
</Tabs>

## <code>get/set</code> Range&lt;int&gt;.is_inclusive {#getset-Range&lt;int&gt;.is_inclusive}

```js
get Range<int>.is_inclusive -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the range is inclusive.
    </TabItem>
</Tabs>

## <code>get/set</code> Range&lt;int&gt;.start {#getset-Range&lt;int&gt;.start}

```js
get Range<int>.start -> int
```

<Tabs>
    <TabItem value="Description" default>

        Return the start of the exclusive range.
    </TabItem>
</Tabs>

## <code>get/set</code> RangeInclusive&lt;int&gt;.end {#getset-RangeInclusive&lt;int&gt;.end}

```js
get RangeInclusive<int>.end -> int
```

<Tabs>
    <TabItem value="Description" default>

        Return the end of the inclusive range.
    </TabItem>
</Tabs>

## <code>get/set</code> RangeInclusive&lt;int&gt;.is_empty {#getset-RangeInclusive&lt;int&gt;.is_empty}

```js
get RangeInclusive<int>.is_empty -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the range contains no items.
    </TabItem>
</Tabs>

## <code>get/set</code> RangeInclusive&lt;int&gt;.is_exclusive {#getset-RangeInclusive&lt;int&gt;.is_exclusive}

```js
get RangeInclusive<int>.is_exclusive -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the range is exclusive.
    </TabItem>
</Tabs>

## <code>get/set</code> RangeInclusive&lt;int&gt;.is_inclusive {#getset-RangeInclusive&lt;int&gt;.is_inclusive}

```js
get RangeInclusive<int>.is_inclusive -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the range is inclusive.
    </TabItem>
</Tabs>

## <code>get/set</code> RangeInclusive&lt;int&gt;.start {#getset-RangeInclusive&lt;int&gt;.start}

```js
get RangeInclusive<int>.start -> int
```

<Tabs>
    <TabItem value="Description" default>

        Return the start of the inclusive range.
    </TabItem>
</Tabs>

## <code>get/set</code> String.bytes {#getset-String.bytes}

```js
get String.bytes -> int
```

<Tabs>
    <TabItem value="Description" default>

        Return the length of the string, in number of bytes used to store it in UTF-8 encoding.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "朝には紅顔ありて夕べには白骨となる";
        
        print(text.bytes);      // prints 51
        ```
    </TabItem>
</Tabs>

## <code>get/set</code> String.chars {#getset-String.chars}

```js
get String.chars -> CharsStream
```

<Tabs>
    <TabItem value="Description" default>

        Return an iterator over all the characters in the string.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        for ch in "hello, world!".chars {"
            print(ch);
        }
        ```
    </TabItem>
</Tabs>

## <code>get/set</code> String.is_empty {#getset-String.is_empty}

```js
get String.is_empty -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the string is empty.
    </TabItem>
</Tabs>

## <code>get/set</code> String.len {#getset-String.len}

```js
get String.len -> int
```

<Tabs>
    <TabItem value="Description" default>

        Return the length of the string, in number of characters.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "朝には紅顔ありて夕べには白骨となる";
        
        print(text.len);        // prints 17
        ```
    </TabItem>
</Tabs>

## <code>fn</code> abs {#fn-abs}

```js
fn abs(x: i128) -> i128
fn abs(x: i8) -> i8
fn abs(x: i32) -> i32
fn abs(x: i16) -> i16
fn abs(x: float) -> float
fn abs(x: f32) -> f32
fn abs(x: int) -> int
```

<Tabs>
    <TabItem value="Description" default>

        Return the absolute value of the number.
    </TabItem>
</Tabs>

## <code>fn</code> acos {#fn-acos}

```js
fn acos(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the arc-cosine of the floating-point number, in radians.
    </TabItem>
</Tabs>

## <code>fn</code> acosh {#fn-acosh}

```js
fn acosh(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the arc-hyperbolic-cosine of the floating-point number, in radians.
    </TabItem>
</Tabs>

## <code>fn</code> all {#fn-all}

```js
fn all(array: Array, filter: String) -> bool
fn all(array: Array, filter: FnPtr) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if all elements in the array return `true` when applied a function named by `filter`.
    </TabItem>
    <TabItem value="Deprecated API" default>


        This method is deprecated and will be removed from the next major version.
        Use `array.all(Fn("fn_name"))` instead.
    </TabItem>
    <TabItem value="Function Parameters" default>


        A function with the same name as the value of `filter` must exist taking these parameters:
        
        * `element`: copy of array element
        * `index` _(optional)_: current index in the array
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = [1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 5];
        
        print(x.all(|v| v > 3));        // prints false
        
        print(x.all(|v| v > 1));        // prints true
        
        print(x.all(|v, i| i > v));     // prints false
        ```
    </TabItem>
</Tabs>

## <code>fn</code> append {#fn-append}

```js
fn append(blob: Blob, value: int)
fn append(string: String, mut item: ?)
fn append(blob: Blob, character: char)
fn append(blob: Blob, string: String)
fn append(array: Array, new_array: Array)
fn append(string: String, utf8: Blob)
fn append(blob1: Blob, blob2: Blob)
```

<Tabs>
    <TabItem value="Description" default>

        Add a new byte `value` to the end of the BLOB.
        
        Only the lower 8 bits of the `value` are used; all other bits are ignored.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let b = blob();
        
        b.push(0x42);
        
        print(b);       // prints "[42]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> as_string {#fn-as_string}

```js
fn as_string(blob: Blob) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Convert the BLOB into a string.
        
        The byte stream must be valid UTF-8, otherwise an error is raised.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let b = blob(5, 0x42);
        
        let x = b.as_string();
        
        print(x);       // prints "FFFFF"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> asin {#fn-asin}

```js
fn asin(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the arc-sine of the floating-point number, in radians.
    </TabItem>
</Tabs>

## <code>fn</code> asinh {#fn-asinh}

```js
fn asinh(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the arc-hyperbolic-sine of the floating-point number, in radians.
    </TabItem>
</Tabs>

## <code>fn</code> atan {#fn-atan}

```js
fn atan(x: float) -> float
fn atan(x: float, y: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the arc-tangent of the floating-point number, in radians.
    </TabItem>
</Tabs>

## <code>fn</code> atanh {#fn-atanh}

```js
fn atanh(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the arc-hyperbolic-tangent of the floating-point number, in radians.
    </TabItem>
</Tabs>

## <code>fn</code> bits {#fn-bits}

```js
fn bits(value: int) -> BitRange
fn bits(value: int, range: RangeInclusive<int>) -> BitRange
fn bits(value: int, range: Range<int>) -> BitRange
fn bits(value: int, from: int) -> BitRange
fn bits(value: int, from: int, len: int) -> BitRange
```

<Tabs>
    <TabItem value="Description" default>

        Return an iterator over all the bits in the number.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = 123456;
        
        for bit in x.bits() {
            print(bit);
        }
        ```
    </TabItem>
</Tabs>

## <code>fn</code> blob {#fn-blob}

```js
fn blob() -> Blob
fn blob(len: int) -> Blob
fn blob(len: int, value: int) -> Blob
```

<Tabs>
    <TabItem value="Description" default>

        Return a new, empty BLOB.
    </TabItem>
</Tabs>

## <code>fn</code> bytes {#fn-bytes}

```js
fn bytes(string: String) -> int
```

<Tabs>
    <TabItem value="Description" default>

        Return the length of the string, in number of bytes used to store it in UTF-8 encoding.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "朝には紅顔ありて夕べには白骨となる";
        
        print(text.bytes);      // prints 51
        ```
    </TabItem>
</Tabs>

## <code>fn</code> ceiling {#fn-ceiling}

```js
fn ceiling(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the smallest whole number larger than or equals to the floating-point number.
    </TabItem>
</Tabs>

## <code>fn</code> chars {#fn-chars}

```js
fn chars(string: String) -> CharsStream
fn chars(string: String, range: Range<int>) -> CharsStream
fn chars(string: String, range: RangeInclusive<int>) -> CharsStream
fn chars(string: String, start: int) -> CharsStream
fn chars(string: String, start: int, len: int) -> CharsStream
```

<Tabs>
    <TabItem value="Description" default>

        Return an iterator over the characters in the string.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        for ch in "hello, world!".chars() {
            print(ch);
        }
        ```
    </TabItem>
</Tabs>

## <code>fn</code> chop {#fn-chop}

```js
fn chop(blob: Blob, len: int)
fn chop(array: Array, len: int)
```

<Tabs>
    <TabItem value="Description" default>

        Cut off the head of the BLOB, leaving a tail of the specified length.
        
        * If `len` ≤ 0, the BLOB is cleared.
        * If `len` ≥ length of BLOB, the BLOB is not modified.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let b = blob();
        
        b += 1; b += 2; b += 3; b += 4; b += 5;
        
        b.chop(3);
        
        print(b);           // prints "[030405]"
        
        b.chop(10);
        
        print(b);           // prints "[030405]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> clear {#fn-clear}

```js
fn clear(array: Array)
fn clear(blob: Blob)
fn clear(string: String)
fn clear(map: Map)
```

<Tabs>
    <TabItem value="Description" default>

        Clear the array.
    </TabItem>
</Tabs>

## <code>fn</code> contains {#fn-contains}

```js
fn contains(string: String, character: char) -> bool
fn contains(range: RangeInclusive<int>, value: int) -> bool
fn contains(array: Array, value: ?) -> bool
fn contains(map: Map, property: String) -> bool
fn contains(blob: Blob, value: int) -> bool
fn contains(string: String, match_string: String) -> bool
fn contains(range: Range<int>, value: int) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the string contains a specified character.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "hello, world!";
        
        print(text.contains('h'));      // prints true
        
        print(text.contains('x'));      // prints false
        ```
    </TabItem>
</Tabs>

## <code>fn</code> cos {#fn-cos}

```js
fn cos(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the cosine of the floating-point number in radians.
    </TabItem>
</Tabs>

## <code>fn</code> cosh {#fn-cosh}

```js
fn cosh(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the hyperbolic cosine of the floating-point number in radians.
    </TabItem>
</Tabs>

## <code>fn</code> crop {#fn-crop}

```js
fn crop(string: String, range: Range<int>)
fn crop(string: String, start: int)
fn crop(string: String, range: RangeInclusive<int>)
fn crop(string: String, start: int, len: int)
```

<Tabs>
    <TabItem value="Description" default>

        Remove all characters from the string except those within an exclusive `range`.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "hello, world!";
        
        text.crop(2..8);
        
        print(text);        // prints "llo, w"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> debug {#fn-debug}

```js
fn debug() -> String
fn debug(string: String) -> String
fn debug(unit: ?) -> String
fn debug(number: float) -> String
fn debug(map: Map) -> String
fn debug(array: Array) -> String
fn debug(number: f32) -> String
fn debug(value: bool) -> String
fn debug(character: char) -> String
fn debug(item: ?) -> String
fn debug(f: FnPtr) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Return the empty string.
    </TabItem>
</Tabs>

## <code>fn</code> dedup {#fn-dedup}

```js
fn dedup(array: Array)
fn dedup(array: Array, comparer: String)
fn dedup(array: Array, comparer: FnPtr)
```

<Tabs>
    <TabItem value="Description" default>

        Remove duplicated _consecutive_ elements from the array.
        
        The operator `==` is used to compare elements and must be defined,
        otherwise `false` is assumed.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = [1, 2, 2, 2, 3, 4, 3, 3, 2, 1];
        
        x.dedup();
        
        print(x);       // prints "[1, 2, 3, 4, 3, 2, 1]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> drain {#fn-drain}

```js
fn drain(blob: Blob, range: RangeInclusive<int>) -> Blob
fn drain(array: Array, filter: String) -> Array
fn drain(array: Array, range: RangeInclusive<int>) -> Array
fn drain(blob: Blob, range: Range<int>) -> Blob
fn drain(map: Map, filter: FnPtr) -> Map
fn drain(array: Array, filter: FnPtr) -> Array
fn drain(array: Array, range: Range<int>) -> Array
fn drain(blob: Blob, start: int, len: int) -> Blob
fn drain(array: Array, start: int, len: int) -> Array
```

<Tabs>
    <TabItem value="Description" default>

        Remove all bytes in the BLOB within an inclusive `range` and return them as a new BLOB.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let b1 = blob();
        
        b1 += 1; b1 += 2; b1 += 3; b1 += 4; b1 += 5;
        
        let b2 = b1.drain(1..=2);
        
        print(b1);      // prints "[010405]"
        
        print(b2);      // prints "[0203]"
        
        let b3 = b1.drain(2..=2);
        
        print(b1);      // prints "[0104]"
        
        print(b3);      // prints "[05]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> elapsed {#fn-elapsed}

```js
fn elapsed(timestamp: Instant) -> ?
```

<Tabs>
    <TabItem value="Description" default>

        Return the number of seconds between the current system time and the timestamp.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let now = timestamp();
        
        sleep(10.0);            // sleep for 10 seconds
        
        print(now.elapsed);     // prints 10.???
        ```
    </TabItem>
</Tabs>

## <code>fn</code> end {#fn-end}

```js
fn end(range: RangeInclusive<int>) -> int
fn end(range: Range<int>) -> int
```

<Tabs>
    <TabItem value="Description" default>

        Return the end of the inclusive range.
    </TabItem>
</Tabs>

## <code>fn</code> ends_with {#fn-ends_with}

```js
fn ends_with(string: String, match_string: String) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the string ends with a specified string.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "hello, world!";
        
        print(text.ends_with("world!"));    // prints true
        
        print(text.ends_with("hello"));     // prints false
        ```
    </TabItem>
</Tabs>

## <code>fn</code> exit {#fn-exit}

```js
fn exit() -> ?
fn exit(value: ?) -> ?
```

<Tabs>
    <TabItem value="Description" default>

        Exit the script evaluation immediately with `()` as exit value.
    </TabItem>
    <TabItem value="Example" default>

        ```rhai
        exit();
        ```
    </TabItem>
</Tabs>

## <code>fn</code> exp {#fn-exp}

```js
fn exp(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the exponential of the floating-point number.
    </TabItem>
</Tabs>

## <code>fn</code> extract {#fn-extract}

```js
fn extract(blob: Blob, range: Range<int>) -> Blob
fn extract(array: Array, range: RangeInclusive<int>) -> Array
fn extract(array: Array, start: int) -> Array
fn extract(blob: Blob, range: RangeInclusive<int>) -> Blob
fn extract(array: Array, range: Range<int>) -> Array
fn extract(blob: Blob, start: int) -> Blob
fn extract(blob: Blob, start: int, len: int) -> Blob
fn extract(array: Array, start: int, len: int) -> Array
```

<Tabs>
    <TabItem value="Description" default>

        Copy an exclusive `range` of the BLOB and return it as a new BLOB.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let b = blob();
        
        b += 1; b += 2; b += 3; b += 4; b += 5;
        
        print(b.extract(1..3));     // prints "[0203]"
        
        print(b);                   // prints "[0102030405]"
        ```
    </TabItem>
</Tabs>

## <code>get/set</code> f32.is_zero {#getset-f32.is_zero}

```js
get f32.is_zero -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the floating-point number is zero.
    </TabItem>
</Tabs>

## <code>fn</code> fill_with {#fn-fill_with}

```js
fn fill_with(map: Map, map2: Map)
```

<Tabs>
    <TabItem value="Description" default>

        Add all property values of another object map into the object map.
        Only properties that do not originally exist in the object map are added.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let m = #{a:1, b:2, c:3};
        let n = #{a: 42, d:0};
        
        m.fill_with(n);
        
        print(m);       // prints "#{a:1, b:2, c:3, d:0}"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> filter {#fn-filter}

```js
fn filter(array: Array, filter: FnPtr) -> Array
fn filter(array: Array, filter_func: String) -> Array
fn filter(map: Map, filter: FnPtr) -> Map
```

<Tabs>
    <TabItem value="Description" default>

        Iterate through all the elements in the array, applying a `filter` function to each element
        in turn, and return a copy of all elements (in order) that return `true` as a new array.
    </TabItem>
    <TabItem value="No Function Parameter" default>


        Array element (mutable) is bound to `this`.
        
        This method is marked _pure_; the `filter` function should not mutate array elements.
    </TabItem>
    <TabItem value="Function Parameters" default>


        * `element`: copy of array element
        * `index` _(optional)_: current index in the array
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = [1, 2, 3, 4, 5];
        
        let y = x.filter(|v| v >= 3);
        
        print(y);       // prints "[3, 4, 5]"
        
        let y = x.filter(|v, i| v * i >= 10);
        
        print(y);       // prints "[12, 20]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> find {#fn-find}

```js
fn find(array: Array, filter: FnPtr) -> ?
fn find(array: Array, filter: FnPtr, start: int) -> ?
```

<Tabs>
    <TabItem value="Description" default>

        Iterate through all the elements in the array, applying a `filter` function to each element
        in turn, and return a copy of the first element that returns `true`. If no element returns
        `true`, `()` is returned.
    </TabItem>
    <TabItem value="No Function Parameter" default>


        Array element (mutable) is bound to `this`.
    </TabItem>
    <TabItem value="Function Parameters" default>


        * `element`: copy of array element
        * `index` _(optional)_: current index in the array
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = [1, 2, 3, 5, 8, 13];
        
        print(x.find(|v| v > 3));                    // prints 5: 5 > 3
        
        print(x.find(|v| v > 13) ?? "not found");    // prints "not found": nothing is > 13
        
        print(x.find(|v, i| v * i > 13));            // prints 5: 3 * 5 > 13
        ```
    </TabItem>
</Tabs>

## <code>fn</code> find_map {#fn-find_map}

```js
fn find_map(array: Array, filter: FnPtr) -> ?
fn find_map(array: Array, filter: FnPtr, start: int) -> ?
```

<Tabs>
    <TabItem value="Description" default>

        Iterate through all the elements in the array, applying a `mapper` function to each element
        in turn, and return the first result that is not `()`. Otherwise, `()` is returned.
    </TabItem>
    <TabItem value="No Function Parameter" default>


        Array element (mutable) is bound to `this`.
        
        This method is marked _pure_; the `mapper` function should not mutate array elements.
    </TabItem>
    <TabItem value="Function Parameters" default>


        * `element`: copy of array element
        * `index` _(optional)_: current index in the array
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = [#{alice: 1}, #{bob: 2}, #{clara: 3}];
        
        print(x.find_map(|v| v.alice));                  // prints 1
        
        print(x.find_map(|v| v.dave) ?? "not found");    // prints "not found"
        
        print(x.find_map(|| this.dave) ?? "not found");  // prints "not found"
        ```
    </TabItem>
</Tabs>

## <code>get/set</code> float.ceiling {#getset-float.ceiling}

```js
get float.ceiling -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the smallest whole number larger than or equals to the floating-point number.
    </TabItem>
</Tabs>

## <code>get/set</code> float.floor {#getset-float.floor}

```js
get float.floor -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the largest whole number less than or equals to the floating-point number.
    </TabItem>
</Tabs>

## <code>get/set</code> float.fraction {#getset-float.fraction}

```js
get float.fraction -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the fractional part of the floating-point number.
    </TabItem>
</Tabs>

## <code>get/set</code> float.int {#getset-float.int}

```js
get float.int -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the integral part of the floating-point number.
    </TabItem>
</Tabs>

## <code>get/set</code> float.is_finite {#getset-float.is_finite}

```js
get float.is_finite -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the floating-point number is finite.
    </TabItem>
</Tabs>

## <code>get/set</code> float.is_infinite {#getset-float.is_infinite}

```js
get float.is_infinite -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the floating-point number is infinite.
    </TabItem>
</Tabs>

## <code>get/set</code> float.is_nan {#getset-float.is_nan}

```js
get float.is_nan -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the floating-point number is `NaN` (Not A Number).
    </TabItem>
</Tabs>

## <code>get/set</code> float.is_zero {#getset-float.is_zero}

```js
get float.is_zero -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the floating-point number is zero.
    </TabItem>
</Tabs>

## <code>get/set</code> float.round {#getset-float.round}

```js
get float.round -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the nearest whole number closest to the floating-point number.
        Rounds away from zero.
    </TabItem>
</Tabs>

## <code>fn</code> floor {#fn-floor}

```js
fn floor(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the largest whole number less than or equals to the floating-point number.
    </TabItem>
</Tabs>

## <code>fn</code> for_each {#fn-for_each}

```js
fn for_each(array: Array, map: FnPtr)
```

<Tabs>
    <TabItem value="Description" default>

        Iterate through all the elements in the array, applying a `process` function to each element in turn.
        Each element is bound to `this` before calling the function.
    </TabItem>
    <TabItem value="Function Parameters" default>


        * `this`: bound to array element (mutable)
        * `index` _(optional)_: current index in the array
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = [1, 2, 3, 4, 5];
        
        x.for_each(|| this *= this);
        
        print(x);       // prints "[1, 4, 9, 16, 25]"
        
        x.for_each(|i| this *= i);
        
        print(x);       // prints "[0, 2, 6, 12, 20]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> fraction {#fn-fraction}

```js
fn fraction(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the fractional part of the floating-point number.
    </TabItem>
</Tabs>

## <code>fn</code> get {#fn-get}

```js
fn get(string: String, index: int) -> ?
fn get(array: Array, index: int) -> ?
fn get(map: Map, property: String) -> ?
fn get(blob: Blob, index: int) -> int
```

<Tabs>
    <TabItem value="Description" default>

        Get the character at the `index` position in the string.
        
        * If `index` < 0, position counts from the end of the string (`-1` is the last character).
        * If `index` < -length of string, zero is returned.
        * If `index` ≥ length of string, zero is returned.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "hello, world!";
        
        print(text.get(0));     // prints 'h'
        
        print(text.get(-1));    // prints '!'
        
        print(text.get(99));    // prints empty (for '()')'
        ```
    </TabItem>
</Tabs>

## <code>fn</code> get_bit {#fn-get_bit}

```js
fn get_bit(value: int, bit: int) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the specified `bit` in the number is set.
        
        If `bit` < 0, position counts from the MSB (Most Significant Bit).
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = 123456;
        
        print(x.get_bit(5));    // prints false
        
        print(x.get_bit(6));    // prints true
        
        print(x.get_bit(-48));  // prints true on 64-bit
        ```
    </TabItem>
</Tabs>

## <code>fn</code> get_bits {#fn-get_bits}

```js
fn get_bits(value: int, range: Range<int>) -> int
fn get_bits(value: int, range: RangeInclusive<int>) -> int
fn get_bits(value: int, start: int, bits: int) -> int
```

<Tabs>
    <TabItem value="Description" default>

        Return an exclusive range of bits in the number as a new number.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = 123456;
        
        print(x.get_bits(5..10));       // print 18
        ```
    </TabItem>
</Tabs>

## <code>fn</code> get_fn_metadata_list {#fn-get_fn_metadata_list}

```js
fn get_fn_metadata_list() -> Array
fn get_fn_metadata_list(name: String) -> Array
fn get_fn_metadata_list(name: String, params: int) -> Array
```

<Tabs>
    <TabItem value="Description" default>

        Return an array of object maps containing metadata of all script-defined functions.
    </TabItem>
</Tabs>

## <code>fn</code> hypot {#fn-hypot}

```js
fn hypot(x: float, y: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the hypotenuse of a triangle with sides `x` and `y`.
    </TabItem>
</Tabs>

## <code>get/set</code> i128.is_even {#getset-i128.is_even}

```js
get i128.is_even -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is even.
    </TabItem>
</Tabs>

## <code>get/set</code> i128.is_odd {#getset-i128.is_odd}

```js
get i128.is_odd -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is odd.
    </TabItem>
</Tabs>

## <code>get/set</code> i128.is_zero {#getset-i128.is_zero}

```js
get i128.is_zero -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is zero.
    </TabItem>
</Tabs>

## <code>get/set</code> i16.is_even {#getset-i16.is_even}

```js
get i16.is_even -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is even.
    </TabItem>
</Tabs>

## <code>get/set</code> i16.is_odd {#getset-i16.is_odd}

```js
get i16.is_odd -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is odd.
    </TabItem>
</Tabs>

## <code>get/set</code> i16.is_zero {#getset-i16.is_zero}

```js
get i16.is_zero -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is zero.
    </TabItem>
</Tabs>

## <code>get/set</code> i32.is_even {#getset-i32.is_even}

```js
get i32.is_even -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is even.
    </TabItem>
</Tabs>

## <code>get/set</code> i32.is_odd {#getset-i32.is_odd}

```js
get i32.is_odd -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is odd.
    </TabItem>
</Tabs>

## <code>get/set</code> i32.is_zero {#getset-i32.is_zero}

```js
get i32.is_zero -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is zero.
    </TabItem>
</Tabs>

## <code>get/set</code> i8.is_even {#getset-i8.is_even}

```js
get i8.is_even -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is even.
    </TabItem>
</Tabs>

## <code>get/set</code> i8.is_odd {#getset-i8.is_odd}

```js
get i8.is_odd -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is odd.
    </TabItem>
</Tabs>

## <code>get/set</code> i8.is_zero {#getset-i8.is_zero}

```js
get i8.is_zero -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is zero.
    </TabItem>
</Tabs>

## <code>fn</code> index_of {#fn-index_of}

```js
fn index_of(string: String, find_string: String) -> int
fn index_of(array: Array, filter: FnPtr) -> int
fn index_of(string: String, character: char) -> int
fn index_of(array: Array, filter: String) -> int
fn index_of(array: Array, value: ?) -> int
fn index_of(string: String, find_string: String, start: int) -> int
fn index_of(array: Array, value: ?, start: int) -> int
fn index_of(array: Array, filter: String, start: int) -> int
fn index_of(string: String, character: char, start: int) -> int
fn index_of(array: Array, filter: FnPtr, start: int) -> int
```

<Tabs>
    <TabItem value="Description" default>

        Find the specified `character` in the string and return the first index where it is found.
        If the `character` is not found, `-1` is returned.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "hello, world! hello, foobar!";
        
        print(text.index_of("ll"));     // prints 2 (first index)
        
        print(text.index_of("xx:));     // prints -1
        ```
    </TabItem>
</Tabs>

## <code>fn</code> insert {#fn-insert}

```js
fn insert(array: Array, index: int, item: ?)
fn insert(blob: Blob, index: int, value: int)
```

<Tabs>
    <TabItem value="Description" default>

        Add a new element into the array at a particular `index` position.
        
        * If `index` < 0, position counts from the end of the array (`-1` is the last element).
        * If `index` < -length of array, the element is added to the beginning of the array.
        * If `index` ≥ length of array, the element is appended to the end of the array.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = [1, 2, 3];
        
        x.insert(0, "hello");
        
        x.insert(2, true);
        
        x.insert(-2, 42);
        
        print(x);       // prints ["hello", 1, true, 2, 42, 3]
        ```
    </TabItem>
</Tabs>

## <code>fn</code> int {#fn-int}

```js
fn int(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the integral part of the floating-point number.
    </TabItem>
</Tabs>

## <code>get/set</code> int.bits {#getset-int.bits}

```js
get int.bits -> BitRange
```

<Tabs>
    <TabItem value="Description" default>

        Return an iterator over all the bits in the number.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = 123456;
        
        for bit in x.bits {
            print(bit);
        }
        ```
    </TabItem>
</Tabs>

## <code>get/set</code> int.is_even {#getset-int.is_even}

```js
get int.is_even -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is even.
    </TabItem>
</Tabs>

## <code>get/set</code> int.is_odd {#getset-int.is_odd}

```js
get int.is_odd -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is odd.
    </TabItem>
</Tabs>

## <code>get/set</code> int.is_zero {#getset-int.is_zero}

```js
get int.is_zero -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is zero.
    </TabItem>
</Tabs>

## <code>fn</code> is_anonymous {#fn-is_anonymous}

```js
fn is_anonymous(fn_ptr: FnPtr) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the function is an anonymous function.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let f = |x| x * 2;
        
        print(f.is_anonymous);      // prints true
        ```
    </TabItem>
</Tabs>

## <code>fn</code> is_empty {#fn-is_empty}

```js
fn is_empty(range: Range<int>) -> bool
fn is_empty(string: String) -> bool
fn is_empty(map: Map) -> bool
fn is_empty(array: Array) -> bool
fn is_empty(range: RangeInclusive<int>) -> bool
fn is_empty(blob: Blob) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the range contains no items.
    </TabItem>
</Tabs>

## <code>fn</code> is_even {#fn-is_even}

```js
fn is_even(x: u32) -> bool
fn is_even(x: int) -> bool
fn is_even(x: u64) -> bool
fn is_even(x: i32) -> bool
fn is_even(x: i16) -> bool
fn is_even(x: i8) -> bool
fn is_even(x: u8) -> bool
fn is_even(x: i128) -> bool
fn is_even(x: u16) -> bool
fn is_even(x: u128) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is even.
    </TabItem>
</Tabs>

## <code>fn</code> is_exclusive {#fn-is_exclusive}

```js
fn is_exclusive(range: RangeInclusive<int>) -> bool
fn is_exclusive(range: Range<int>) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the range is exclusive.
    </TabItem>
</Tabs>

## <code>fn</code> is_finite {#fn-is_finite}

```js
fn is_finite(x: float) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the floating-point number is finite.
    </TabItem>
</Tabs>

## <code>fn</code> is_inclusive {#fn-is_inclusive}

```js
fn is_inclusive(range: RangeInclusive<int>) -> bool
fn is_inclusive(range: Range<int>) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the range is inclusive.
    </TabItem>
</Tabs>

## <code>fn</code> is_infinite {#fn-is_infinite}

```js
fn is_infinite(x: float) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the floating-point number is infinite.
    </TabItem>
</Tabs>

## <code>fn</code> is_nan {#fn-is_nan}

```js
fn is_nan(x: float) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the floating-point number is `NaN` (Not A Number).
    </TabItem>
</Tabs>

## <code>fn</code> is_odd {#fn-is_odd}

```js
fn is_odd(x: u32) -> bool
fn is_odd(x: int) -> bool
fn is_odd(x: i16) -> bool
fn is_odd(x: i32) -> bool
fn is_odd(x: u64) -> bool
fn is_odd(x: u8) -> bool
fn is_odd(x: i8) -> bool
fn is_odd(x: u16) -> bool
fn is_odd(x: i128) -> bool
fn is_odd(x: u128) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is odd.
    </TabItem>
</Tabs>

## <code>fn</code> is_zero {#fn-is_zero}

```js
fn is_zero(x: f32) -> bool
fn is_zero(x: int) -> bool
fn is_zero(x: u32) -> bool
fn is_zero(x: u128) -> bool
fn is_zero(x: i128) -> bool
fn is_zero(x: u16) -> bool
fn is_zero(x: i8) -> bool
fn is_zero(x: u8) -> bool
fn is_zero(x: i16) -> bool
fn is_zero(x: float) -> bool
fn is_zero(x: u64) -> bool
fn is_zero(x: i32) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the floating-point number is zero.
    </TabItem>
</Tabs>

## <code>fn</code> keys {#fn-keys}

```js
fn keys(map: Map) -> Array
```

<Tabs>
    <TabItem value="Description" default>

        Return an array with all the property names in the object map.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let m = #{a:1, b:2, c:3};
        
        print(m.keys());        // prints ["a", "b", "c"]
        ```
    </TabItem>
</Tabs>

## <code>fn</code> len {#fn-len}

```js
fn len(map: Map) -> int
fn len(string: String) -> int
fn len(array: Array) -> int
fn len(blob: Blob) -> int
```

<Tabs>
    <TabItem value="Description" default>

        Return the number of properties in the object map.
    </TabItem>
</Tabs>

## <code>fn</code> ln {#fn-ln}

```js
fn ln(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the natural log of the floating-point number.
    </TabItem>
</Tabs>

## <code>fn</code> log {#fn-log}

```js
fn log(x: float) -> float
fn log(x: float, base: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the log of the floating-point number with base 10.
    </TabItem>
</Tabs>

## <code>fn</code> make_lower {#fn-make_lower}

```js
fn make_lower(character: char)
fn make_lower(string: String)
```

<Tabs>
    <TabItem value="Description" default>

        Convert the character to lower-case.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let ch = 'A';
        
        ch.make_lower();
        
        print(ch);          // prints 'a'
        ```
    </TabItem>
</Tabs>

## <code>fn</code> make_upper {#fn-make_upper}

```js
fn make_upper(string: String)
fn make_upper(character: char)
```

<Tabs>
    <TabItem value="Description" default>

        Convert the string to all upper-case.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "hello, world!"
        
        text.make_upper();
        
        print(text);        // prints "HELLO, WORLD!";
        ```
    </TabItem>
</Tabs>

## <code>fn</code> map {#fn-map}

```js
fn map(array: Array, map: FnPtr) -> Array
fn map(array: Array, mapper: String) -> Array
```

<Tabs>
    <TabItem value="Description" default>

        Iterate through all the elements in the array, applying a `mapper` function to each element
        in turn, and return the results as a new array.
    </TabItem>
    <TabItem value="No Function Parameter" default>


        Array element (mutable) is bound to `this`.
        
        This method is marked _pure_; the `mapper` function should not mutate array elements.
    </TabItem>
    <TabItem value="Function Parameters" default>


        * `element`: copy of array element
        * `index` _(optional)_: current index in the array
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = [1, 2, 3, 4, 5];
        
        let y = x.map(|v| v * v);
        
        print(y);       // prints "[1, 4, 9, 16, 25]"
        
        let y = x.map(|v, i| v * i);
        
        print(y);       // prints "[0, 2, 6, 12, 20]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> max {#fn-max}

```js
fn max(x: f32, y: float) -> float
fn max(x: f32, y: int) -> f32
fn max(x: int, y: float) -> float
fn max(x: int, y: f32) -> f32
fn max(x: float, y: int) -> float
fn max(x: i8, y: i8) -> i8
fn max(x: u16, y: u16) -> u16
fn max(x: u64, y: u64) -> u64
fn max(x: i16, y: i16) -> i16
fn max(x: i128, y: i128) -> i128
fn max(x: u128, y: u128) -> u128
fn max(x: int, y: int) -> int
fn max(x: i32, y: i32) -> i32
fn max(x: u32, y: u32) -> u32
fn max(x: f32, y: f32) -> f32
fn max(x: float, y: float) -> float
fn max(x: u8, y: u8) -> u8
fn max(string1: String, string2: String) -> String
fn max(x: float, y: f32) -> float
fn max(char1: char, char2: char) -> char
```

<Tabs>
    <TabItem value="Description" default>

        Return the number that is larger than the other number.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        max(42, 123);   // returns 132
        ```
    </TabItem>
</Tabs>

## <code>fn</code> min {#fn-min}

```js
fn min(x: int, y: f32) -> f32
fn min(x: f32, y: float) -> float
fn min(x: f32, y: int) -> f32
fn min(x: int, y: float) -> float
fn min(x: u64, y: u64) -> u64
fn min(x: u16, y: u16) -> u16
fn min(x: float, y: int) -> float
fn min(x: i8, y: i8) -> i8
fn min(x: int, y: int) -> int
fn min(x: u128, y: u128) -> u128
fn min(x: i32, y: i32) -> i32
fn min(x: u32, y: u32) -> u32
fn min(x: f32, y: f32) -> f32
fn min(x: float, y: float) -> float
fn min(x: i16, y: i16) -> i16
fn min(string1: String, string2: String) -> String
fn min(x: i128, y: i128) -> i128
fn min(x: u8, y: u8) -> u8
fn min(x: float, y: f32) -> float
fn min(char1: char, char2: char) -> char
```

<Tabs>
    <TabItem value="Description" default>

        Return the number that is smaller than the other number.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        min(42, 123);   // returns 42
        ```
    </TabItem>
</Tabs>

## <code>fn</code> mixin {#fn-mixin}

```js
fn mixin(map: Map, map2: Map)
```

<Tabs>
    <TabItem value="Description" default>

        Add all property values of another object map into the object map.
        Existing property values of the same names are replaced.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let m = #{a:1, b:2, c:3};
        let n = #{a: 42, d:0};
        
        m.mixin(n);
        
        print(m);       // prints "#{a:42, b:2, c:3, d:0}"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> name {#fn-name}

```js
fn name(fn_ptr: FnPtr) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Return the name of the function.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        fn double(x) { x * 2 }
        
        let f = Fn("double");
        
        print(f.name);      // prints "double"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> pad {#fn-pad}

```js
fn pad(blob: Blob, len: int, value: int)
fn pad(string: String, len: int, character: char)
fn pad(array: Array, len: int, item: ?)
fn pad(string: String, len: int, padding: String)
```

<Tabs>
    <TabItem value="Description" default>

        Pad the BLOB to at least the specified length with copies of a specified byte `value`.
        
        If `len` ≤ length of BLOB, no padding is done.
        
        Only the lower 8 bits of the `value` are used; all other bits are ignored.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let b = blob(3, 0x42);
        
        b.pad(5, 0x18)
        
        print(b);               // prints "[4242421818]"
        
        b.pad(3, 0xab)
        
        print(b);               // prints "[4242421818]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> parse_be_float {#fn-parse_be_float}

```js
fn parse_be_float(blob: Blob, range: Range<int>) -> float
fn parse_be_float(blob: Blob, range: RangeInclusive<int>) -> float
fn parse_be_float(blob: Blob, start: int, len: int) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Parse the bytes within an exclusive `range` in the BLOB as a `FLOAT`
        in big-endian byte order.
        
        * If number of bytes in `range` < number of bytes for `FLOAT`, zeros are padded.
        * If number of bytes in `range` > number of bytes for `FLOAT`, extra bytes are ignored.
    </TabItem>
</Tabs>

## <code>fn</code> parse_be_int {#fn-parse_be_int}

```js
fn parse_be_int(blob: Blob, range: RangeInclusive<int>) -> int
fn parse_be_int(blob: Blob, range: Range<int>) -> int
fn parse_be_int(blob: Blob, start: int, len: int) -> int
```

<Tabs>
    <TabItem value="Description" default>

        Parse the bytes within an inclusive `range` in the BLOB as an `INT`
        in big-endian byte order.
        
        * If number of bytes in `range` < number of bytes for `INT`, zeros are padded.
        * If number of bytes in `range` > number of bytes for `INT`, extra bytes are ignored.
        
        ```rhai
        let b = blob();
        
        b += 1; b += 2; b += 3; b += 4; b += 5;
        
        let x = b.parse_be_int(1..=3);  // parse three bytes
        
        print(x.to_hex());              // prints "0203040000...00"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> parse_float {#fn-parse_float}

```js
fn parse_float(string: String) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Parse a string into a floating-point number.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = parse_int("123.456");
        
        print(x);       // prints 123.456
        ```
    </TabItem>
</Tabs>

## <code>fn</code> parse_int {#fn-parse_int}

```js
fn parse_int(string: String) -> int
fn parse_int(string: String, radix: int) -> int
```

<Tabs>
    <TabItem value="Description" default>

        Parse a string into an integer number.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = parse_int("123");
        
        print(x);       // prints 123
        ```
    </TabItem>
</Tabs>

## <code>fn</code> parse_json {#fn-parse_json}

```js
fn parse_json(json: String) -> ?
```

<Tabs>
    <TabItem value="Description" default>

        Parse a JSON string into a value.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let m = parse_json(`{"a":1, "b":2, "c":3}`);
        
        print(m);       // prints #{"a":1, "b":2, "c":3}
        ```
    </TabItem>
</Tabs>

## <code>fn</code> parse_le_float {#fn-parse_le_float}

```js
fn parse_le_float(blob: Blob, range: Range<int>) -> float
fn parse_le_float(blob: Blob, range: RangeInclusive<int>) -> float
fn parse_le_float(blob: Blob, start: int, len: int) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Parse the bytes within an exclusive `range` in the BLOB as a `FLOAT`
        in little-endian byte order.
        
        * If number of bytes in `range` < number of bytes for `FLOAT`, zeros are padded.
        * If number of bytes in `range` > number of bytes for `FLOAT`, extra bytes are ignored.
    </TabItem>
</Tabs>

## <code>fn</code> parse_le_int {#fn-parse_le_int}

```js
fn parse_le_int(blob: Blob, range: RangeInclusive<int>) -> int
fn parse_le_int(blob: Blob, range: Range<int>) -> int
fn parse_le_int(blob: Blob, start: int, len: int) -> int
```

<Tabs>
    <TabItem value="Description" default>

        Parse the bytes within an inclusive `range` in the BLOB as an `INT`
        in little-endian byte order.
        
        * If number of bytes in `range` < number of bytes for `INT`, zeros are padded.
        * If number of bytes in `range` > number of bytes for `INT`, extra bytes are ignored.
        
        ```rhai
        let b = blob();
        
        b += 1; b += 2; b += 3; b += 4; b += 5;
        
        let x = b.parse_le_int(1..=3);  // parse three bytes
        
        print(x.to_hex());              // prints "040302"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> pop {#fn-pop}

```js
fn pop(blob: Blob) -> int
fn pop(array: Array) -> ?
fn pop(string: String) -> ?
fn pop(string: String, len: int) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Remove the last byte from the BLOB and return it.
        
        If the BLOB is empty, zero is returned.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let b = blob();
        
        b += 1; b += 2; b += 3; b += 4; b += 5;
        
        print(b.pop());         // prints 5
        
        print(b);               // prints "[01020304]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> print {#fn-print}

```js
fn print() -> String
fn print(number: float) -> String
fn print(map: Map) -> String
fn print(string: String) -> String
fn print(unit: ?) -> String
fn print(item: ?) -> String
fn print(array: Array) -> String
fn print(number: f32) -> String
fn print(value: bool) -> String
fn print(character: char) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Return the empty string.
    </TabItem>
</Tabs>

## <code>fn</code> push {#fn-push}

```js
fn push(array: Array, item: ?)
fn push(blob: Blob, value: int)
```

<Tabs>
    <TabItem value="Description" default>

        Add a new element, which is not another array, to the end of the array.
        
        If `item` is `Array`, then `append` is more specific and will be called instead.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = [1, 2, 3];
        
        x.push("hello");
        
        print(x);       // prints [1, 2, 3, "hello"]
        ```
    </TabItem>
</Tabs>

## <code>fn</code> range {#fn-range}

```js
fn range(from: u64, to: u64) -> Range<u64>
fn range(from: u16, to: u16) -> Range<u16>
fn range(range: Range<u128>, step: u128) -> StepRange<u128>
fn range(range: Range<u8>, step: u8) -> StepRange<u8>
fn range(from: i8, to: i8) -> Range<i8>
fn range(range: Range<u16>, step: u16) -> StepRange<u16>
fn range(range: Range<i128>, step: i128) -> StepRange<i128>
fn range(from: u8, to: u8) -> Range<u8>
fn range(range: Range<i16>, step: i16) -> StepRange<i16>
fn range(range: Range<i32>, step: i32) -> StepRange<i32>
fn range(range: Range<float>, step: float) -> StepRange<float>
fn range(from: u32, to: u32) -> Range<u32>
fn range(from: i32, to: i32) -> Range<i32>
fn range(from: int, to: int) -> Range<int>
fn range(from: u128, to: u128) -> Range<u128>
fn range(range: Range<int>, step: int) -> StepRange<int>
fn range(range: Range<u32>, step: u32) -> StepRange<u32>
fn range(range: Range<u64>, step: u64) -> StepRange<u64>
fn range(from: i128, to: i128) -> Range<i128>
fn range(range: Range<i8>, step: i8) -> StepRange<i8>
fn range(from: i16, to: i16) -> Range<i16>
fn range(from: i16, to: i16, step: i16) -> StepRange<i16>
fn range(from: i128, to: i128, step: i128) -> StepRange<i128>
fn range(from: i8, to: i8, step: i8) -> StepRange<i8>
fn range(from: float, to: float, step: float) -> StepRange<float>
fn range(from: u16, to: u16, step: u16) -> StepRange<u16>
fn range(from: u128, to: u128, step: u128) -> StepRange<u128>
fn range(from: u8, to: u8, step: u8) -> StepRange<u8>
fn range(from: i32, to: i32, step: i32) -> StepRange<i32>
fn range(from: int, to: int, step: int) -> StepRange<int>
fn range(from: u64, to: u64, step: u64) -> StepRange<u64>
fn range(from: u32, to: u32, step: u32) -> StepRange<u32>
```

<Tabs>
    <TabItem value="Description" default>

        Return an iterator over the exclusive range of `from..to`.
        The value `to` is never included.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        // prints all values from 8 to 17
        for n in range(8, 18) {
            print(n);
        }
        ```
    </TabItem>
</Tabs>

## <code>fn</code> reduce {#fn-reduce}

```js
fn reduce(array: Array, reducer: String) -> ?
fn reduce(array: Array, reducer: FnPtr) -> ?
fn reduce(array: Array, reducer: FnPtr, initial: ?) -> ?
fn reduce(array: Array, reducer: String, initial: ?) -> ?
```

<Tabs>
    <TabItem value="Description" default>

        Reduce an array by iterating through all elements while applying a function named by `reducer`.
    </TabItem>
    <TabItem value="Deprecated API" default>


        This method is deprecated and will be removed from the next major version.
        Use `array.reduce(Fn("fn_name"))` instead.
    </TabItem>
    <TabItem value="Function Parameters" default>


        A function with the same name as the value of `reducer` must exist taking these parameters:
        
        * `result`: accumulated result, initially `()`
        * `element`: copy of array element
        * `index` _(optional)_: current index in the array
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        fn process(r, x) {
            x + (r ?? 0)
        }
        fn process_extra(r, x, i) {
            x + i + (r ?? 0)
        }
        
        let x = [1, 2, 3, 4, 5];
        
        let y = x.reduce("process");
        
        print(y);       // prints 15
        
        let y = x.reduce("process_extra");
        
        print(y);       // prints 25
        ```
    </TabItem>
</Tabs>

## <code>fn</code> reduce_rev {#fn-reduce_rev}

```js
fn reduce_rev(array: Array, reducer: String) -> ?
fn reduce_rev(array: Array, reducer: FnPtr) -> ?
fn reduce_rev(array: Array, reducer: String, initial: ?) -> ?
fn reduce_rev(array: Array, reducer: FnPtr, initial: ?) -> ?
```

<Tabs>
    <TabItem value="Description" default>

        Reduce an array by iterating through all elements, in _reverse_ order,
        while applying a function named by `reducer`.
    </TabItem>
    <TabItem value="Deprecated API" default>


        This method is deprecated and will be removed from the next major version.
        Use `array.reduce_rev(Fn("fn_name"))` instead.
    </TabItem>
    <TabItem value="Function Parameters" default>


        A function with the same name as the value of `reducer` must exist taking these parameters:
        
        * `result`: accumulated result, initially `()`
        * `element`: copy of array element
        * `index` _(optional)_: current index in the array
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        fn process(r, x) {
            x + (r ?? 0)
        }
        fn process_extra(r, x, i) {
            x + i + (r ?? 0)
        }
        
        let x = [1, 2, 3, 4, 5];
        
        let y = x.reduce_rev("process");
        
        print(y);       // prints 15
        
        let y = x.reduce_rev("process_extra");
        
        print(y);       // prints 25
        ```
    </TabItem>
</Tabs>

## <code>fn</code> remove {#fn-remove}

```js
fn remove(map: Map, property: String) -> ?
fn remove(array: Array, index: int) -> ?
fn remove(string: String, character: char)
fn remove(string: String, sub_string: String)
fn remove(blob: Blob, index: int) -> int
```

<Tabs>
    <TabItem value="Description" default>

        Remove any property of the specified `name` from the object map, returning its value.
        
        If the property does not exist, `()` is returned.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let m = #{a:1, b:2, c:3};
        
        let x = m.remove("b");
        
        print(x);       // prints 2
        
        print(m);       // prints "#{a:1, c:3}"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> replace {#fn-replace}

```js
fn replace(string: String, find_character: char, substitute_string: String)
fn replace(string: String, find_string: String, substitute_string: String)
fn replace(string: String, find_string: String, substitute_character: char)
fn replace(string: String, find_character: char, substitute_character: char)
```

<Tabs>
    <TabItem value="Description" default>

        Replace all occurrences of the specified character in the string with another string.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "hello, world! hello, foobar!";
        
        text.replace('l', "(^)");
        
        print(text);        // prints "he(^)(^)o, wor(^)d! he(^)(^)o, foobar!"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> retain {#fn-retain}

```js
fn retain(array: Array, range: Range<int>) -> Array
fn retain(array: Array, filter: FnPtr) -> Array
fn retain(array: Array, range: RangeInclusive<int>) -> Array
fn retain(map: Map, filter: FnPtr) -> Map
fn retain(blob: Blob, range: Range<int>) -> Blob
fn retain(array: Array, filter: String) -> Array
fn retain(blob: Blob, range: RangeInclusive<int>) -> Blob
fn retain(array: Array, start: int, len: int) -> Array
fn retain(blob: Blob, start: int, len: int) -> Blob
```

<Tabs>
    <TabItem value="Description" default>

        Remove all elements in the array not within an exclusive `range` and return them as a new array.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = [1, 2, 3, 4, 5];
        
        let y = x.retain(1..4);
        
        print(x);       // prints "[2, 3, 4]"
        
        print(y);       // prints "[1, 5]"
        
        let z = x.retain(1..3);
        
        print(x);       // prints "[3, 4]"
        
        print(z);       // prints "[1]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> reverse {#fn-reverse}

```js
fn reverse(blob: Blob)
fn reverse(array: Array)
```

<Tabs>
    <TabItem value="Description" default>

        Reverse the BLOB.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let b = blob();
        
        b += 1; b += 2; b += 3; b += 4; b += 5;
        
        print(b);           // prints "[0102030405]"
        
        b.reverse();
        
        print(b);           // prints "[0504030201]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> round {#fn-round}

```js
fn round(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the nearest whole number closest to the floating-point number.
        Rounds away from zero.
    </TabItem>
</Tabs>

## <code>fn</code> set {#fn-set}

```js
fn set(string: String, index: int, character: char)
fn set(blob: Blob, index: int, value: int)
fn set(map: Map, property: String, value: ?)
fn set(array: Array, index: int, value: ?)
```

<Tabs>
    <TabItem value="Description" default>

        Set the `index` position in the string to a new `character`.
        
        * If `index` < 0, position counts from the end of the string (`-1` is the last character).
        * If `index` < -length of string, the string is not modified.
        * If `index` ≥ length of string, the string is not modified.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "hello, world!";
        
        text.set(3, 'x');
        
        print(text);     // prints "helxo, world!"
        
        text.set(-3, 'x');
        
        print(text);    // prints "hello, worxd!"
        
        text.set(99, 'x');
        
        print(text);    // prints "hello, worxd!"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> set_bit {#fn-set_bit}

```js
fn set_bit(value: int, bit: int, new_value: bool)
```

<Tabs>
    <TabItem value="Description" default>

        Set the specified `bit` in the number if the new value is `true`.
        Clear the `bit` if the new value is `false`.
        
        If `bit` < 0, position counts from the MSB (Most Significant Bit).
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = 123456;
        
        x.set_bit(5, true);
        
        print(x);               // prints 123488
        
        x.set_bit(6, false);
        
        print(x);               // prints 123424
        
        x.set_bit(-48, false);
        
        print(x);               // prints 57888 on 64-bit
        ```
    </TabItem>
</Tabs>

## <code>fn</code> set_bits {#fn-set_bits}

```js
fn set_bits(value: int, range: RangeInclusive<int>, new_value: int)
fn set_bits(value: int, range: Range<int>, new_value: int)
fn set_bits(value: int, bit: int, bits: int, new_value: int)
```

<Tabs>
    <TabItem value="Description" default>

        Replace an inclusive range of bits in the number with a new value.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = 123456;
        
        x.set_bits(5..=9, 42);
        
        print(x);           // print 123200
        ```
    </TabItem>
</Tabs>

## <code>fn</code> set_tag {#fn-set_tag}

```js
fn set_tag(value: ?, tag: int)
```

<Tabs>
    <TabItem value="Description" default>

        Set the _tag_ of a `Dynamic` value.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = "hello, world!";
        
        x.tag = 42;
        
        print(x.tag);           // prints 42
        ```
    </TabItem>
</Tabs>

## <code>fn</code> shift {#fn-shift}

```js
fn shift(blob: Blob) -> int
fn shift(array: Array) -> ?
```

<Tabs>
    <TabItem value="Description" default>

        Remove the first byte from the BLOB and return it.
        
        If the BLOB is empty, zero is returned.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let b = blob();
        
        b += 1; b += 2; b += 3; b += 4; b += 5;
        
        print(b.shift());       // prints 1
        
        print(b);               // prints "[02030405]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> sign {#fn-sign}

```js
fn sign(x: int) -> int
fn sign(x: f32) -> int
fn sign(x: i32) -> int
fn sign(x: float) -> int
fn sign(x: i16) -> int
fn sign(x: i8) -> int
fn sign(x: i128) -> int
```

<Tabs>
    <TabItem value="Description" default>

        Return the sign (as an integer) of the number according to the following:
        
        * `0` if the number is zero
        * `1` if the number is positive
        * `-1` if the number is negative
    </TabItem>
</Tabs>

## <code>fn</code> sin {#fn-sin}

```js
fn sin(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the sine of the floating-point number in radians.
    </TabItem>
</Tabs>

## <code>fn</code> sinh {#fn-sinh}

```js
fn sinh(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the hyperbolic sine of the floating-point number in radians.
    </TabItem>
</Tabs>

## <code>fn</code> sleep {#fn-sleep}

```js
fn sleep(seconds: float)
fn sleep(seconds: int)
```

<Tabs>
    <TabItem value="Description" default>

        Block the current thread for a particular number of `seconds`.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        // Do nothing for 10 seconds!
        sleep(10.0);
        ```
    </TabItem>
</Tabs>

## <code>fn</code> some {#fn-some}

```js
fn some(array: Array, filter: String) -> bool
fn some(array: Array, filter: FnPtr) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if any element in the array that returns `true` when applied a function named
        by `filter`.
    </TabItem>
    <TabItem value="Deprecated API" default>


        This method is deprecated and will be removed from the next major version.
        Use `array.some(Fn("fn_name"))` instead.
    </TabItem>
    <TabItem value="Function Parameters" default>


        A function with the same name as the value of `filter` must exist taking these parameters:
        
        * `element`: copy of array element
        * `index` _(optional)_: current index in the array
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        fn large(x) { x > 3 }
        
        fn huge(x) { x > 10 }
        
        fn screen(x, i) { i > x }
        
        let x = [1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 5];
        
        print(x.some("large"));     // prints true
        
        print(x.some("huge"));      // prints false
        
        print(x.some("screen"));    // prints true
        ```
    </TabItem>
</Tabs>

## <code>fn</code> sort {#fn-sort}

```js
fn sort(array: Array)
fn sort(array: Array, comparer: String)
fn sort(array: Array, comparer: FnPtr)
```

<Tabs>
    <TabItem value="Description" default>

        Sort the array.
        
        All elements in the array must be of the same data type.
    </TabItem>
    <TabItem value="Supported Data Types" default>


        * integer numbers
        * floating-point numbers
        * decimal numbers
        * characters
        * strings
        * booleans
        * `()`
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = [1, 3, 5, 7, 9, 2, 4, 6, 8, 10];
        
        x.sort();
        
        print(x);       // prints "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> splice {#fn-splice}

```js
fn splice(blob: Blob, range: Range<int>, replace: Blob)
fn splice(array: Array, range: RangeInclusive<int>, replace: Array)
fn splice(blob: Blob, range: RangeInclusive<int>, replace: Blob)
fn splice(array: Array, range: Range<int>, replace: Array)
fn splice(blob: Blob, start: int, len: int, replace: Blob)
fn splice(array: Array, start: int, len: int, replace: Array)
```

<Tabs>
    <TabItem value="Description" default>

        Replace an exclusive `range` of the BLOB with another BLOB.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let b1 = blob(10, 0x42);
        let b2 = blob(5, 0x18);
        
        b1.splice(1..4, b2);
        
        print(b1);      // prints "[4218181818184242 42424242]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> split {#fn-split}

```js
fn split(string: String) -> Array
fn split(string: String, delimiter: String) -> Array
fn split(blob: Blob, index: int) -> Blob
fn split(array: Array, index: int) -> Array
fn split(string: String, index: int) -> Array
fn split(string: String, delimiter: char) -> Array
fn split(string: String, delimiter: String, segments: int) -> Array
fn split(string: String, delimiter: char, segments: int) -> Array
```

<Tabs>
    <TabItem value="Description" default>

        Split the string into segments based on whitespaces, returning an array of the segments.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "hello, world! hello, foo!";
        
        print(text.split());        // prints ["hello,", "world!", "hello,", "foo!"]
        ```
    </TabItem>
</Tabs>

## <code>fn</code> split_rev {#fn-split_rev}

```js
fn split_rev(string: String, delimiter: char) -> Array
fn split_rev(string: String, delimiter: String) -> Array
fn split_rev(string: String, delimiter: String, segments: int) -> Array
fn split_rev(string: String, delimiter: char, segments: int) -> Array
```

<Tabs>
    <TabItem value="Description" default>

        Split the string into segments based on a `delimiter` character, returning an array of
        the segments in _reverse_ order.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "hello, world! hello, foo!";
        
        print(text.split_rev('l'));     // prints ["o, foo!", "", "d! he", "o, wor", "", "he"]
        ```
    </TabItem>
</Tabs>

## <code>fn</code> sqrt {#fn-sqrt}

```js
fn sqrt(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the square root of the floating-point number.
    </TabItem>
</Tabs>

## <code>fn</code> start {#fn-start}

```js
fn start(range: RangeInclusive<int>) -> int
fn start(range: Range<int>) -> int
```

<Tabs>
    <TabItem value="Description" default>

        Return the start of the inclusive range.
    </TabItem>
</Tabs>

## <code>fn</code> starts_with {#fn-starts_with}

```js
fn starts_with(string: String, match_string: String) -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return `true` if the string starts with a specified string.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "hello, world!";
        
        print(text.starts_with("hello"));   // prints true
        
        print(text.starts_with("world"));   // prints false
        ```
    </TabItem>
</Tabs>

## <code>fn</code> sub_string {#fn-sub_string}

```js
fn sub_string(string: String, range: Range<int>) -> String
fn sub_string(string: String, range: RangeInclusive<int>) -> String
fn sub_string(string: String, start: int) -> String
fn sub_string(string: String, start: int, len: int) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Copy an exclusive range of characters from the string and return it as a new string.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "hello, world!";
        
        print(text.sub_string(3..7));   // prints "lo, "
        ```
    </TabItem>
</Tabs>

## <code>fn</code> tag {#fn-tag}

```js
fn tag(value: ?) -> int
```

<Tabs>
    <TabItem value="Description" default>

        Return the _tag_ of a `Dynamic` value.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = "hello, world!";
        
        x.tag = 42;
        
        print(x.tag);           // prints 42
        ```
    </TabItem>
</Tabs>

## <code>fn</code> take {#fn-take}

```js
fn take(value: ?) -> ?
```

<Tabs>
    <TabItem value="Description" default>

        Take ownership of the data in a `Dynamic` value and return it.
        The data is _NOT_ cloned.
        
        The original value is replaced with `()`.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = 42;
        
        print(take(x));         // prints 42
        
        print(x);               // prints ()
        ```
    </TabItem>
</Tabs>

## <code>fn</code> tan {#fn-tan}

```js
fn tan(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the tangent of the floating-point number in radians.
    </TabItem>
</Tabs>

## <code>fn</code> tanh {#fn-tanh}

```js
fn tanh(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Return the hyperbolic tangent of the floating-point number in radians.
    </TabItem>
</Tabs>

## <code>fn</code> timestamp {#fn-timestamp}

```js
fn timestamp() -> Instant
```

<Tabs>
    <TabItem value="Description" default>

        Create a timestamp containing the current system time.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let now = timestamp();
        
        sleep(10.0);            // sleep for 10 seconds
        
        print(now.elapsed);     // prints 10.???
        ```
    </TabItem>
</Tabs>

## <code>fn</code> to_array {#fn-to_array}

```js
fn to_array(blob: Blob) -> Array
```

<Tabs>
    <TabItem value="Description" default>

        Convert the BLOB into an array of integers.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let b = blob(5, 0x42);
        
        let x = b.to_array();
        
        print(x);       // prints "[66, 66, 66, 66, 66]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> to_binary {#fn-to_binary}

```js
fn to_binary(value: int) -> String
fn to_binary(value: u32) -> String
fn to_binary(value: i128) -> String
fn to_binary(value: u16) -> String
fn to_binary(value: u128) -> String
fn to_binary(value: i16) -> String
fn to_binary(value: u64) -> String
fn to_binary(value: i32) -> String
fn to_binary(value: i8) -> String
fn to_binary(value: u8) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Convert the `value` into a string in binary format.
    </TabItem>
</Tabs>

## <code>fn</code> to_blob {#fn-to_blob}

```js
fn to_blob(string: String) -> Blob
```

<Tabs>
    <TabItem value="Description" default>

        Convert the string into an UTF-8 encoded byte-stream as a BLOB.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "朝には紅顔ありて夕べには白骨となる";
        
        let bytes = text.to_blob();
        
        print(bytes.len());     // prints 51
        ```
    </TabItem>
</Tabs>

## <code>fn</code> to_chars {#fn-to_chars}

```js
fn to_chars(string: String) -> Array
```

<Tabs>
    <TabItem value="Description" default>

        Return an array containing all the characters of the string.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "hello";
        
        print(text.to_chars());     // prints "['h', 'e', 'l', 'l', 'o']"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> to_debug {#fn-to_debug}

```js
fn to_debug(item: ?) -> String
fn to_debug(f: FnPtr) -> String
fn to_debug(array: Array) -> String
fn to_debug(value: bool) -> String
fn to_debug(character: char) -> String
fn to_debug(number: f32) -> String
fn to_debug(map: Map) -> String
fn to_debug(number: float) -> String
fn to_debug(string: String) -> String
fn to_debug(unit: ?) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Convert the value of the `item` into a string in debug format.
    </TabItem>
</Tabs>

## <code>fn</code> to_degrees {#fn-to_degrees}

```js
fn to_degrees(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Convert radians to degrees.
    </TabItem>
</Tabs>

## <code>fn</code> to_float {#fn-to_float}

```js
fn to_float(x: f32) -> float
fn to_float()
fn to_float()
fn to_float()
fn to_float()
fn to_float()
fn to_float()
fn to_float()
fn to_float()
fn to_float()
fn to_float()
fn to_float()
```

<Tabs>
    <TabItem value="Description" default>

        Convert the 32-bit floating-point number to 64-bit.
    </TabItem>
</Tabs>

## <code>fn</code> to_hex {#fn-to_hex}

```js
fn to_hex(value: u32) -> String
fn to_hex(value: int) -> String
fn to_hex(value: i16) -> String
fn to_hex(value: u64) -> String
fn to_hex(value: i32) -> String
fn to_hex(value: i8) -> String
fn to_hex(value: u8) -> String
fn to_hex(value: i128) -> String
fn to_hex(value: u16) -> String
fn to_hex(value: u128) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Convert the `value` into a string in hex format.
    </TabItem>
</Tabs>

## <code>fn</code> to_int {#fn-to_int}

```js
fn to_int()
fn to_int()
fn to_int()
fn to_int()
fn to_int()
fn to_int()
fn to_int()
fn to_int(x: float) -> int
fn to_int()
fn to_int()
fn to_int(x: f32) -> int
fn to_int()
fn to_int()
```

<Tabs>
    <TabItem value="Description" default>

        Convert the floating-point number into an integer.
    </TabItem>
</Tabs>

## <code>fn</code> to_json {#fn-to_json}

```js
fn to_json(map: Map) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Return the JSON representation of the object map.
    </TabItem>
    <TabItem value="Data types" default>


        Only the following data types should be kept inside the object map:
        `INT`, `FLOAT`, `ImmutableString`, `char`, `bool`, `()`, `Array`, `Map`.
    </TabItem>
    <TabItem value="Errors" default>


        Data types not supported by JSON serialize into formats that may
        invalidate the result.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let m = #{a:1, b:2, c:3};
        
        print(m.to_json());     // prints {"a":1, "b":2, "c":3}
        ```
    </TabItem>
</Tabs>

## <code>fn</code> to_lower {#fn-to_lower}

```js
fn to_lower(string: String) -> String
fn to_lower(character: char) -> char
```

<Tabs>
    <TabItem value="Description" default>

        Convert the string to all lower-case and return it as a new string.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "HELLO, WORLD!"
        
        print(text.to_lower());     // prints "hello, world!"
        
        print(text);                // prints "HELLO, WORLD!"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> to_octal {#fn-to_octal}

```js
fn to_octal(value: int) -> String
fn to_octal(value: u32) -> String
fn to_octal(value: u8) -> String
fn to_octal(value: i8) -> String
fn to_octal(value: i16) -> String
fn to_octal(value: i32) -> String
fn to_octal(value: u64) -> String
fn to_octal(value: u128) -> String
fn to_octal(value: u16) -> String
fn to_octal(value: i128) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Convert the `value` into a string in octal format.
    </TabItem>
</Tabs>

## <code>fn</code> to_radians {#fn-to_radians}

```js
fn to_radians(x: float) -> float
```

<Tabs>
    <TabItem value="Description" default>

        Convert degrees to radians.
    </TabItem>
</Tabs>

## <code>fn</code> to_string {#fn-to_string}

```js
fn to_string(item: ?) -> String
fn to_string(array: Array) -> String
fn to_string(value: bool) -> String
fn to_string(character: char) -> String
fn to_string(number: f32) -> String
fn to_string(map: Map) -> String
fn to_string(number: float) -> String
fn to_string(string: String) -> String
fn to_string(unit: ?) -> String
```

<Tabs>
    <TabItem value="Description" default>

        Convert the value of the `item` into a string.
    </TabItem>
</Tabs>

## <code>fn</code> to_upper {#fn-to_upper}

```js
fn to_upper(string: String) -> String
fn to_upper(character: char) -> char
```

<Tabs>
    <TabItem value="Description" default>

        Convert the string to all upper-case and return it as a new string.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "hello, world!"
        
        print(text.to_upper());     // prints "HELLO, WORLD!"
        
        print(text);                // prints "hello, world!"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> trim {#fn-trim}

```js
fn trim(string: String)
```

<Tabs>
    <TabItem value="Description" default>

        Remove whitespace characters from both ends of the string.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "   hello     ";
        
        text.trim();
        
        print(text);    // prints "hello"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> truncate {#fn-truncate}

```js
fn truncate(string: String, len: int)
fn truncate(array: Array, len: int)
fn truncate(blob: Blob, len: int)
```

<Tabs>
    <TabItem value="Description" default>

        Cut off the string at the specified number of characters.
        
        * If `len` ≤ 0, the string is cleared.
        * If `len` ≥ length of string, the string is not truncated.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let text = "hello, world! hello, foobar!";
        
        text.truncate(13);
        
        print(text);    // prints "hello, world!"
        
        text.truncate(10);
        
        print(text);    // prints "hello, world!"
        ```
    </TabItem>
</Tabs>

## <code>get/set</code> u128.is_even {#getset-u128.is_even}

```js
get u128.is_even -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is even.
    </TabItem>
</Tabs>

## <code>get/set</code> u128.is_odd {#getset-u128.is_odd}

```js
get u128.is_odd -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is odd.
    </TabItem>
</Tabs>

## <code>get/set</code> u128.is_zero {#getset-u128.is_zero}

```js
get u128.is_zero -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is zero.
    </TabItem>
</Tabs>

## <code>get/set</code> u16.is_even {#getset-u16.is_even}

```js
get u16.is_even -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is even.
    </TabItem>
</Tabs>

## <code>get/set</code> u16.is_odd {#getset-u16.is_odd}

```js
get u16.is_odd -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is odd.
    </TabItem>
</Tabs>

## <code>get/set</code> u16.is_zero {#getset-u16.is_zero}

```js
get u16.is_zero -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is zero.
    </TabItem>
</Tabs>

## <code>get/set</code> u32.is_even {#getset-u32.is_even}

```js
get u32.is_even -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is even.
    </TabItem>
</Tabs>

## <code>get/set</code> u32.is_odd {#getset-u32.is_odd}

```js
get u32.is_odd -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is odd.
    </TabItem>
</Tabs>

## <code>get/set</code> u32.is_zero {#getset-u32.is_zero}

```js
get u32.is_zero -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is zero.
    </TabItem>
</Tabs>

## <code>get/set</code> u64.is_even {#getset-u64.is_even}

```js
get u64.is_even -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is even.
    </TabItem>
</Tabs>

## <code>get/set</code> u64.is_odd {#getset-u64.is_odd}

```js
get u64.is_odd -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is odd.
    </TabItem>
</Tabs>

## <code>get/set</code> u64.is_zero {#getset-u64.is_zero}

```js
get u64.is_zero -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is zero.
    </TabItem>
</Tabs>

## <code>get/set</code> u8.is_even {#getset-u8.is_even}

```js
get u8.is_even -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is even.
    </TabItem>
</Tabs>

## <code>get/set</code> u8.is_odd {#getset-u8.is_odd}

```js
get u8.is_odd -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is odd.
    </TabItem>
</Tabs>

## <code>get/set</code> u8.is_zero {#getset-u8.is_zero}

```js
get u8.is_zero -> bool
```

<Tabs>
    <TabItem value="Description" default>

        Return true if the number is zero.
    </TabItem>
</Tabs>

## <code>fn</code> values {#fn-values}

```js
fn values(map: Map) -> Array
```

<Tabs>
    <TabItem value="Description" default>

        Return an array with all the property values in the object map.
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let m = #{a:1, b:2, c:3};
        
        print(m.values());      // prints "[1, 2, 3]""
        ```
    </TabItem>
</Tabs>

## <code>fn</code> write_ascii {#fn-write_ascii}

```js
fn write_ascii(blob: Blob, range: Range<int>, string: String)
fn write_ascii(blob: Blob, range: RangeInclusive<int>, string: String)
fn write_ascii(blob: Blob, start: int, len: int, string: String)
```

<Tabs>
    <TabItem value="Description" default>

        Write an ASCII string to the bytes within an exclusive `range` in the BLOB.
        
        Each ASCII character encodes to one single byte in the BLOB.
        Non-ASCII characters are ignored.
        
        * If number of bytes in `range` < length of `string`, extra bytes in `string` are not written.
        * If number of bytes in `range` > length of `string`, extra bytes in `range` are not modified.
        
        ```rhai
        let b = blob(8);
        
        b.write_ascii(1..5, "hello, world!");
        
        print(b);       // prints "[0068656c6c000000]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> write_be {#fn-write_be}

```js
fn write_be(blob: Blob, range: RangeInclusive<int>, value: int)
fn write_be(blob: Blob, range: Range<int>, value: int)
fn write_be(blob: Blob, range: Range<int>, value: float)
fn write_be(blob: Blob, range: RangeInclusive<int>, value: float)
fn write_be(blob: Blob, start: int, len: int, value: float)
fn write_be(blob: Blob, start: int, len: int, value: int)
```

<Tabs>
    <TabItem value="Description" default>

        Write an `INT` value to the bytes within an inclusive `range` in the BLOB
        in big-endian byte order.
        
        * If number of bytes in `range` < number of bytes for `INT`, extra bytes in `INT` are not written.
        * If number of bytes in `range` > number of bytes for `INT`, extra bytes in `range` are not modified.
        
        ```rhai
        let b = blob(8, 0x42);
        
        b.write_be_int(1..=3, 0x99);
        
        print(b);       // prints "[4200000042424242]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> write_le {#fn-write_le}

```js
fn write_le(blob: Blob, range: Range<int>, value: float)
fn write_le(blob: Blob, range: RangeInclusive<int>, value: float)
fn write_le(blob: Blob, range: RangeInclusive<int>, value: int)
fn write_le(blob: Blob, range: Range<int>, value: int)
fn write_le(blob: Blob, start: int, len: int, value: int)
fn write_le(blob: Blob, start: int, len: int, value: float)
```

<Tabs>
    <TabItem value="Description" default>

        Write a `FLOAT` value to the bytes within an exclusive `range` in the BLOB
        in little-endian byte order.
        
        * If number of bytes in `range` < number of bytes for `FLOAT`, extra bytes in `FLOAT` are not written.
        * If number of bytes in `range` > number of bytes for `FLOAT`, extra bytes in `range` are not modified.
    </TabItem>
</Tabs>

## <code>fn</code> write_utf8 {#fn-write_utf8}

```js
fn write_utf8(blob: Blob, range: Range<int>, string: String)
fn write_utf8(blob: Blob, range: RangeInclusive<int>, string: String)
fn write_utf8(blob: Blob, start: int, len: int, string: String)
```

<Tabs>
    <TabItem value="Description" default>

        Write a string to the bytes within an exclusive `range` in the BLOB in UTF-8 encoding.
        
        * If number of bytes in `range` < length of `string`, extra bytes in `string` are not written.
        * If number of bytes in `range` > length of `string`, extra bytes in `range` are not modified.
        
        ```rhai
        let b = blob(8);
        
        b.write_utf8(1..5, "朝には紅顔ありて夕べには白骨となる");
        
        print(b);       // prints "[00e69c9de3000000]"
        ```
    </TabItem>
</Tabs>

## <code>fn</code> zip {#fn-zip}

```js
fn zip(array1: Array, array2: Array, map: FnPtr) -> Array
```

<Tabs>
    <TabItem value="Description" default>

        Iterate through all elements in two arrays, applying a `mapper` function to them,
        and return a new array containing the results.
    </TabItem>
    <TabItem value="Function Parameters" default>


        * `array1`: First array
        * `array2`: Second array
        * `index` _(optional)_: current index in the array
    </TabItem>
    <TabItem value="Example" default>


        ```rhai
        let x = [1, 2, 3, 4, 5];
        let y = [9, 8, 7, 6];
        
        let z = x.zip(y, |a, b| a + b);
        
        print(z);       // prints [10, 10, 10, 10]
        
        let z = x.zip(y, |a, b, i| a + b + i);
        
        print(z);       // prints [10, 11, 12, 13]
        ```
    </TabItem>
</Tabs>
