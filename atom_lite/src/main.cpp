#include <ArduinoJson.h>  //https://github.com/bblanchon/ArduinoJson (use v6.xx)
#include <M5Atom.h>
#include <PubSubClient.h>
#include <WiFi.h>
#include <WiFiClientSecure.h>

#include "ATOM_PRINTER.h"
#define emptyString String()

// Follow instructions from
// https://github.com/debsahu/ESP-MQTT-AWS-IoT-Core/blob/master/doc/README.md
// Enter values in secrets.h ▼
#include "secrets.h"
const char *MQTT_SUB_TOPIC = "volp/share/link";
const int MQTT_PORT = 8883;

WiFiClientSecure net;

PubSubClient client(net);

unsigned long lastMillis = 0;

ATOM_PRINTER printer;

CRGB dispColor(uint8_t r, uint8_t g, uint8_t b) {
    return (CRGB)((r << 16) | (g << 8) | b);
}

void volp_print(const char *status) {
    Serial.println("Printing QR Code:");
    Serial.println(status);

    printer.init();
    printer.newLine(5);
    printer.init();
    printer.printQRCode(status);
    printer.init();
    printer.newLine(1);
    printer.init();
    printer.printASCII("Message Received!");
    printer.init();
    printer.newLine(10);

    delay(2000);
}

void messageReceived(char *topic, byte *payload, unsigned int length) {
    Serial.println("Message arrived");
    M5.dis.drawpix(0, dispColor(255, 255, 0));

    // Convert byte* payload to String
    String payload_str = String((char *)payload).substring(0, length);

    // Deserialize JSON
    DynamicJsonDocument jsonDoc(128);
    DeserializationError error = deserializeJson(jsonDoc, payload_str);

    if (error) {
        // If there's an error in parsing, display it on the screen
        Serial.println("Failed to parse JSON");
        M5.dis.drawpix(0, 0xffff00);
        Serial.println(error.c_str());
        return;
    }
    // Read 'status' from JSON
    const char *status = jsonDoc["link"];

    // ここでstatusにQRコードのURLが入っている
    volp_print(status);

    M5.dis.drawpix(0, dispColor(255, 255, 0));
}

void pubSubErr(int8_t MQTTErr) {
    if (MQTTErr == MQTT_CONNECTION_TIMEOUT)
        Serial.print("Connection tiemout");
    else if (MQTTErr == MQTT_CONNECTION_LOST)
        Serial.print("Connection lost");
    else if (MQTTErr == MQTT_CONNECT_FAILED)
        Serial.print("Connect failed");
    else if (MQTTErr == MQTT_DISCONNECTED)
        Serial.print("Disconnected");
    else if (MQTTErr == MQTT_CONNECTED)
        Serial.print("Connected");
    else if (MQTTErr == MQTT_CONNECT_BAD_PROTOCOL)
        Serial.print("Connect bad protocol");
    else if (MQTTErr == MQTT_CONNECT_BAD_CLIENT_ID)
        Serial.print("Connect bad Client-ID");
    else if (MQTTErr == MQTT_CONNECT_UNAVAILABLE)
        Serial.print("Connect unavailable");
    else if (MQTTErr == MQTT_CONNECT_BAD_CREDENTIALS)
        Serial.print("Connect bad credentials");
    else if (MQTTErr == MQTT_CONNECT_UNAUTHORIZED)
        Serial.print("Connect unauthorized");
}

void connectToMqtt(bool nonBlocking = false) {
    Serial.print("MQTT connecting ");
    M5.dis.drawpix(0, dispColor(255, 255, 0));
    while (!client.connected()) {
        if (client.connect(THINGNAME)) {
            Serial.println("connected!");
            if (!client.subscribe(MQTT_SUB_TOPIC)) pubSubErr(client.state());
        } else {
            M5.dis.drawpix(0, dispColor(255, 0, 0));
            Serial.print("failed, reason -> ");
            pubSubErr(client.state());
            if (!nonBlocking) {
                Serial.println(" < try again in 5 seconds");
                delay(5000);
            } else {
                Serial.println(" <");
            }
        }
        if (nonBlocking) break;
    }
    M5.dis.drawpix(0, dispColor(0, 255, 0));
}

void connectToWiFi(String init_str) {
    if (init_str != emptyString) Serial.print(init_str);
    while (WiFi.status() != WL_CONNECTED) {
        Serial.print(".");
        M5.dis.drawpix(0, dispColor(0, 0, 255));
        delay(1000);
        M5.dis.drawpix(0, dispColor(0, 0, 0));
        delay(1000);
    }
    if (init_str != emptyString) Serial.println("ok!");
}

void checkWiFiThenMQTT(void) {
    connectToWiFi("Checking WiFi");
    connectToMqtt();
}

unsigned long previousMillis = 0;
const long interval = 5000;

void checkWiFiThenMQTTNonBlocking(void) {
    connectToWiFi(emptyString);
    if (millis() - previousMillis >= interval && !client.connected()) {
        previousMillis = millis();
        connectToMqtt(true);
    }
}

void checkWiFiThenReboot(void) {
    connectToWiFi("Checking WiFi");
    Serial.print("Rebooting");
    ESP.restart();
}

void setup() {
    M5.begin(true, false, true);
    Serial.begin(115200);
    printer.begin();
    printer.init();
    delay(5000);

    Serial.println();
    Serial.println();
    WiFi.setHostname(THINGNAME);
    WiFi.mode(WIFI_STA);
    WiFi.begin(WIFI_SSID, WIFI_PASSWORD);
    connectToWiFi(String("Attempting to connect to SSID: ") +
                  String(WIFI_SSID));

    net.setCACert(AWS_CERT_CA);
    net.setCertificate(AWS_CERT_CRT);
    net.setPrivateKey(AWS_CERT_PRIVATE);

    client.setServer(AWS_IOT_ENDPOINT, MQTT_PORT);
    client.setCallback(messageReceived);

    connectToMqtt();
}

void loop() {
    if (!client.connected()) {
        checkWiFiThenMQTT();
        // checkWiFiThenMQTTNonBlocking();
        // checkWiFiThenReboot();
    } else {
        client.loop();
        if (millis() - lastMillis > 5000) {
            lastMillis = millis();
        }
    }
}
