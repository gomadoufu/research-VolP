
#include "PrinterApi.h"
#include "mybitmap.h"
#include <ArduinoJson.h>
#include <M5Atom.h>
#include <PubSubClient.h>
#include <WiFi.h>

// CPSLAB_minimini.png
// https://m5stack.lang-ship.com/tools/image2data/

const uint16_t imgWidth = 104;
const uint16_t imgHeight = 141;

// 1bit Dump

Printer atomPrinter;
HardwareSerial AtomSerial(1);

#define SERVER "mqtt.beebotte.com" // Domain name of Beebotte MQTT service

// Enter your WiFi credentials
const char *ssid = "CPSLAB_WLX";
const char *password = "6bepa8ideapbu";

// to track delay since last reconnection
int64_t lastReconnectAttempt = 0;

WiFiClient wifiClient;
PubSubClient client(wifiClient);

#define TOKEN "token_reIEJ1cjo8Wz12sj" // Set your channel token here
#define CHANNEL "CPSLAB"               // Replace with your device name
#define TOPIC "opencampus" // ここがMQTTのトピックであることに注意

void onMessage(char *_topic, byte *payload, unsigned int length);

void Print_BMP(int width, int height, const unsigned char *data, int mode,
               int wait) {
  AtomSerial.write(0x1D);
  AtomSerial.write(0x76);
  AtomSerial.write(0x30);                     // 0
  AtomSerial.write(mode);                     // m
  AtomSerial.write((width / 8) & 0xff);       // xL
  AtomSerial.write((width / 256 / 8) & 0xff); // xH
  AtomSerial.write((height)&0xff);            // yL
  AtomSerial.write((height / 256) & 0xff);    // yH
  for (int i = 0; i < (width / 8 * height); i++) {
    AtomSerial.write(data[i]); // data
    delay(wait);
  }
}

void setup() {
  M5.begin(true, false, true);
  Serial.begin(115200);
  atomPrinter.Set_Printer_Uart(AtomSerial, 23, 33, 9600);
  atomPrinter.Printer_Init();
  atomPrinter.NewLine_Setting(0x0A);

  /** Wifiの設定 **/
  Serial.println("Connecting to WiFi...");
  M5.dis.drawpix(0, 0x00ffff);
  WiFi.begin(ssid, password);
  while (WiFi.status() != WL_CONNECTED) {
    Serial.println("WiFi Connection Failed, Retrying...");
    M5.dis.drawpix(0, 0xff0000);
    delay(1000);
  }
  Serial.println("Connected to WiFi.");
  M5.dis.drawpix(0, 0x00ff00);

  /** MQTT Client Serverの設定 **/
  client.setServer(SERVER, 1883);
  client.setCallback(onMessage);

  // give the WiFi a second to initialize:
  delay(1000);
  lastReconnectAttempt = 0;
}

/** MQTTメッセージ受信時のコールバック関数 **/
void onMessage(char *_topic, byte *payload, unsigned int length) {
  Serial.println("Message arrived");
  M5.dis.drawpix(0, 0x00ff00);

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
  const char *status = jsonDoc["data"];

  // ここでstatusにQRコードのURLが入っている
  Serial.println(status);

  atomPrinter.Printer_Init();
  atomPrinter.Print_NewLine(2);
  //   atomPrinter.Printer_Init();
  //   Print_BMP(352, 350, iwaiimg, 0x30, 10);
  atomPrinter.Printer_Init();
  atomPrinter.Print_ASCII("TDU\n");
  atomPrinter.Print_ASCII("Open Campus 2023\n");
  atomPrinter.Print_ASCII("CPSLAB:\n");
  atomPrinter.Print_ASCII("Cyber Physical System Lab\n");
  atomPrinter.Print_NewLine(2);
  atomPrinter.Printer_Init();
  atomPrinter.Set_adjlevel("M");
  atomPrinter.Set_QRCode(status);
  atomPrinter.Print_QRCode();
  atomPrinter.Print_NewLine(2);

  //   printer.init();
  //   printer.printASCII("TDUCPSLAB");
  //   printer.printBMP(0, 32, 32, img);
  //   printer.printQRCode(status);
  //   printer.printASCII("\n \n \n \n \n");
  delay(3000);
}

/** MQTT接続用関数 **/
boolean reconnect() {
  uint64_t chipid = ESP.getEfuseMac();
  String clientId = "ESP32-" + String((uint16_t)(chipid >> 32), HEX);
  Serial.println("Connecting to MQTT...");
  M5.dis.drawpix(0, 0x00ffff);
  if (client.connect(clientId.c_str(), TOKEN, "")) {
    Serial.println("MQTT connected");
    M5.dis.drawpix(0, 0x00ff00);
    char topic[64];
    sprintf(topic, "%s/%s", CHANNEL, TOPIC);
    client.subscribe(topic);
  } else {
    Serial.println("MQTT Connection Failed, Retrying...");
    M5.dis.drawpix(0, 0xff0000);
    Serial.println(client.state());
  }
  return client.connected();
}

void loop() {
  if (!client.connected()) {
    int64_t now = millis();
    if (now - lastReconnectAttempt > 5000) {
      lastReconnectAttempt = now;
      // Attempt to reconnect
      if (reconnect()) {
        lastReconnectAttempt = 0;
      }
    }
  } else {
    // Client connected
    delay(50);
    client.loop();
  }
}
