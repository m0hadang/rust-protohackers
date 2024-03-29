```
1: Prime Time
View leaderboard
Fri, 26 Aug 2022 02:00:00

To keep costs down, a hot new government department is contracting out its mission-critical primality testing to the lowest bidder. (That's you).

Officials have devised a JSON-based request-response protocol. Each request is a single line containing a JSON object, terminated by a newline character ('\n', or ASCII 10). Each request begets a response, which is also a single line containing a JSON object, terminated by a newline character.

After connecting, a client may send multiple requests in a single session. Each request should be handled in order.

A conforming request object has the required field method, which must always contain the string "isPrime", and the required field number, which must contain a number. Any JSON number is a valid number, including floating-point values.

Example request:

{"method":"isPrime","number":123}
A request is malformed if it is not a well-formed JSON object, if any required field is missing, if the method name is not "isPrime", or if the number value is not a number.

Extraneous fields are to be ignored.

A conforming response object has the required field method, which must always contain the string "isPrime", and the required field prime, which must contain a boolean value: true if the number in the request was prime, false if it was not.

Example response:

{"method":"isPrime","prime":false}
A response is malformed if it is not a well-formed JSON object, if any required field is missing, if the method name is not "isPrime", or if the prime value is not a boolean.

A response object is considered incorrect if it is well-formed but has an incorrect prime value. Note that non-integers can not be prime.

Accept TCP connections.

Whenever you receive a conforming request, send back a correct response, and wait for another request.

Whenever you receive a malformed request, send back a single malformed response, and disconnect the client.

Make sure you can handle at least 5 simultaneous clients.
```

# Error Handle
- Extraneous fields are to be ignored.
  - 관계 없는 필드는 무시
- A request is malformed if it is not a well-formed JSON object, if any required field is missing, if the method name is not "isPrime", or if the number value is not a number.
  - malformed request는 유효하지 않은 request이다.
  - 필요한 필드가 없는 경우
  - 필드의 이름이 정확히지 않은 경우
  - 필드의 타입이 부정확한 경우
- Whenever you receive a malformed request, send back a single malformed response, and disconnect the client.
  - malformed request를 받을 경우 malformed repose를 반환하고 세션을 끝는다.
- Make sure you can handle at least 5 simultaneous clients.
  - 동시에 5 클라이언트 처리 가능