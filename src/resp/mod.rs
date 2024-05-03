/*
-如何解析Frame
    - simple string:"+OK\r\n"
    - error:"-Error message\r\n"
    - bulk error:"!<Length>\r\n<error>\r\n"
    - integer:"[>]<value>\r\n"
    - bulk string:"$<Length>\r\n<data>\r\n"
    - null bulk string:"$-1\r\n"
    - array:"*<number-of-elements>\r\n<element-1>...<element-n>"
        -"*2\r\n$3\r\nget\r\n$5\r\nhellolr\n"
    - null array:"*-1\r\n"
    - null:"_\r\n"
    - boolean:"#<tf>\r\n"
    - double:",[+>]<integral>[.<fractional>][<Ee>[sign]<exponent>]\r\n"
    - big number:"([+]<number>\r\n"
    - map:"%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
    - set:"~<number-of-elements>\r\n<element-1>..<element-n>"
*/

/*
思路，创建一个RedisFrame的枚举，用来保存所有的数据类型
创建一个Encoding和Decoding的trait，用来表示如何处理数据
为每一个类型 实现encoding和decoding的trait
*/

pub struct RespFrame {}
