.pio/libdeps/PubSubClient/src/PubSubClient.h  
にある、MAX_PACKET_SIZE を大きくすること！！！

```
// MQTT_MAX_PACKET_SIZE : Maximum packet size. Override with setBufferSize().
#ifndef MQTT_MAX_PACKET_SIZE
// #define MQTT_MAX_PACKET_SIZE 256
#define MQTT_MAX_PACKET_SIZE 2048
#endif
```
