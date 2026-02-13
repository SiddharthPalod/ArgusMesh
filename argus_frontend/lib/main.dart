import 'package:flutter/material.dart';
import 'package:argus_frontend/pages/myapp.dart';

Future<void> main() async {
  print("FLUTTER STARTUP: ensureInitialized");
  WidgetsFlutterBinding.ensureInitialized();
  
  print("FLUTTER STARTUP: runApp");
  runApp(const MyApp());
}
