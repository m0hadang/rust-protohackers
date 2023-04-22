# Message format
```
Byte:  |  0  |  1     2     3     4  |  5     6     7     8  |
Type:  |char |         int32         |         int32         |
```

Each message from a client is 9 bytes long
- 각 메시지는 9 바이트

Messages are not delimited by newlines or any other character: you'll know where one message ends and the next starts because they are always 9 bytes.
- 문자(\n 포함)들로 메시지를 구분하지 않는다


The first byte of a message is a character indicating its type. This will be an ASCII uppercase 'I' or 'Q' character, indicating whether the message inserts or queries prices, respectively.
- 첫 1 바이트 문자는 I, Q가 온다

The next 8 bytes are two signed two's complement 32-bit integers in network byte order (big endian), whose meaning depends on the message type. We'll refer to these numbers as int32, but note this may differ from your system's native int32 type (if any), particularly with regard to byte order.
- 이후 8 바이트는 int32 정수형, 네트워크 오더링 데이터 온다

# Insert
```
Byte:  |  0  |  1     2     3     4  |  5     6     7     8  |
Type:  |char |         int32         |         int32         |
Value: | 'I' |       timestamp       |         price         |
```
Insertions may occur out-of-order.
- 삽입은 시간 순서가 아닐 수 있다
While rare, prices can go negative.
- price 는 음수가 될 수 있다.
Behaviour is undefined if there are multiple prices with the same timestamp from the same client.
- 같은 클라이언트로부터 같은 시간대에 여러 가격이 오는것은 정의되지 않았다.

# Query
```
Byte:  |  0  |  1     2     3     4  |  5     6     7     8  |
Type:  |char |         int32         |         int32         |
Value: | 'Q' |        mintime        |        maxtime        |
```

The server must compute the mean of the inserted prices with timestamps T, mintime <= T <= maxtime (i.e. timestamps in the closed interval [mintime, maxtime]). If the mean is not an integer, it is acceptable to round either up or down, at the server's discretion.
- mintime 과 maxtime 사이의 입금 평균을 구해서 반환
- 평균은 서버에서 내림, 올림 처리 하며 반환 할때는 int32 정수형 데이터로 반환
```
Hexadecimal: 00 00 13 f3
Decoded:            5107
```

If there are no samples within the requested period, or if mintime comes after maxtime, the value returned must be 0.
- 주어진 시간 범위에 데이터가 없거나 mintime이 maxtime을 넘어서는 경우 0 반환

# Example session
```
    Hexadecimal:                 Decoded:
<-- 49 00 00 30 39 00 00 00 65   I 12345 101
<-- 49 00 00 30 3a 00 00 00 66   I 12346 102
<-- 49 00 00 30 3b 00 00 00 64   I 12347 100
<-- 49 00 00 a0 00 00 00 00 05   I 40960 5
<-- 51 00 00 30 00 00 00 40 00   Q 12288 16384
--> 00 00 00 65                  101
```

# Other requirements
Make sure you can handle at least 5 simultaneous clients.
- 동시에 5 클라이언트 처리

Where a client triggers undefined behaviour, the server can do anything it likes for that client, but must not adversely affect other clients that did not trigger undefined behaviour.
- 클라이언트가 정의되지 않은 동작을 할 경우 서버는 클라이언트에게 어떤 처리를 하여도 괜찮으나 다른 클라이언트에 영향을 주면 안됨