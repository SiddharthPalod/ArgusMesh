/// Model representing the current state of a mesh node.
class NodeStateModel {
  final String? status;
  final Map<String, dynamic> rawData;

  NodeStateModel({
    this.status,
    required this.rawData,
  });

  factory NodeStateModel.fromMap(Map<String, dynamic> map) {
    return NodeStateModel(
      status: map['status']?.toString(),
      rawData: map,
    );
  }

  String get displayStatus => status ?? '—';
}
