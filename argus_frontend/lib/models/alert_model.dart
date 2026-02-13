import 'dart:convert';

/// Model representing an alert in the mesh network.
class AlertModel {
  final String? senderId;
  final String? sender;
  final String? priority;
  final String? msgId;
  final Map<String, dynamic> payload;

  AlertModel({
    this.senderId,
    this.sender,
    this.priority,
    this.msgId,
    required this.payload,
  });

  factory AlertModel.fromMap(Map<String, dynamic> map) {
    Map<String, dynamic> parsedPayload = {};
    
    // Handle payload - it might be a JSON string or a Map
    if (map['payload'] != null) {
      if (map['payload'] is String) {
        try {
          parsedPayload = jsonDecode(map['payload'] as String) as Map<String, dynamic>;
        } catch (_) {
          // If parsing fails, use empty map
          parsedPayload = {};
        }
      } else if (map['payload'] is Map) {
        parsedPayload = Map<String, dynamic>.from(map['payload'] as Map);
      }
    }

    return AlertModel(
      senderId: map['sender_id']?.toString(),
      sender: map['sender']?.toString(),
      priority: map['priority']?.toString(),
      msgId: map['msg_id']?.toString(),
      payload: parsedPayload,
    );
  }

  String get displaySender => senderId ?? sender ?? '?';
  String get displayPriority => priority ?? '?';
  String get displayMsgId => msgId ?? '—';
}
