import 'package:flutter/material.dart';
import 'package:argus_frontend/models/alert_model.dart';

/// Widget displaying a list of alerts.
/// Single Responsibility: Display alerts list only.
class AlertsList extends StatelessWidget {
  final List<dynamic> alerts;

  const AlertsList({
    super.key,
    required this.alerts,
  });

  Color _alertColor(String priority) {
    switch (priority.toLowerCase()) {
      case 'critical':
        return Colors.red;
      case 'high':
        return Colors.orange;
      case 'normal':
        return Colors.blue;
      default:
        return Colors.grey;
    }
  }

  @override
  Widget build(BuildContext context) {
    if (alerts.isEmpty) {
      return const Card(
        child: Padding(
          padding: EdgeInsets.all(24),
          child: Center(
            child: Text('No alerts yet'),
          ),
        ),
      );
    }

    return Column(
      children: alerts.map((a) {
        final map = a is Map ? a as Map<String, dynamic> : <String, dynamic>{};
        final alert = AlertModel.fromMap(map);
        final priority = alert.displayPriority;

        return Card(
          margin: const EdgeInsets.only(bottom: 8),
          child: ListTile(
            leading: CircleAvatar(
              backgroundColor: _alertColor(priority),
              child: Text(
                priority.isNotEmpty ? priority[0] : '?',
                style: const TextStyle(
                  color: Colors.white,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
            title: Text(alert.displaySender),
            subtitle: Text(alert.displayMsgId),
          ),
        );
      }).toList(),
    );
  }
}
