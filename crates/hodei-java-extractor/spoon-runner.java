import spoon.Launcher;
import spoon.support.compiler.jdt.JDTCompiler;
import java.io.FileWriter;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.stream.Collectors;
import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

/**
 * Spoon Runner - Main entry point for Spoon analysis
 *
 * This Java program runs Spoon analysis on source files and outputs
 * semantic model as JSON for consumption by Rust.
 */
public class SpoonRunner {
    private static class SemanticModel {
        String package;
        String className;
        String superClass;
        java.util.List<String> interfaces;
        java.util.List<MethodInfo> methods;
        java.util.List<FieldInfo> fields;
        java.util.List<String> annotations;

        static class MethodInfo {
            String name;
            String signature;
            String returnType;
            java.util.List<String> parameters;
            boolean isPublic;
            boolean isPrivate;
            boolean isProtected;
            boolean isStatic;
            boolean isAbstract;
            java.util.List<String> callSites;
            String body;
        }

        static class FieldInfo {
            String name;
            String type;
            boolean isPublic;
            boolean isPrivate;
            boolean isProtected;
            boolean isStatic;
            boolean isFinal;
            String initializer;
        }
    }

    public static void main(String[] args) {
        if (args.length < 1) {
            System.err.println("Usage: SpoonRunner <source-path> [output-file]");
            System.exit(1);
        }

        String sourcePath = args[0];
        String outputFile = args.length > 1 ? args[1] : "spoon-output.json";

        try {
            // Create Spoon launcher
            Launcher launcher = new Launcher();

            // Add source path
            launcher.addInputPath(sourcePath);

            // Build model
            launcher.buildModel();

            // Process all classes
            var factory = launcher.getFactory();
            var classes = factory.Class().getAll();

            java.util.List<SemanticModel> models = new java.util.ArrayList<>();

            for (var ctClass : classes) {
                if (ctClass.isPrimitive() || ctClass.isAnonymous() || ctClass.isLocalClass()) {
                    continue;
                }

                SemanticModel model = new SemanticModel();
                model.package = ctClass.getPackage() != null ? ctClass.getPackage().getQualifiedName() : "";
                model.className = ctClass.getSimpleName();
                model.superClass = ctClass.getSuperclass() != null ? ctClass.getSuperclass().getQualifiedName() : null;

                // Interfaces
                model.interfaces = ctClass.getSuperInterfaces().stream()
                    .map(i -> i.getQualifiedName())
                    .collect(Collectors.toList());

                // Methods
                model.methods = new java.util.ArrayList<>();
                for (var ctMethod : ctClass.getMethods()) {
                    if (ctMethod.isImplicit()) continue;

                    SemanticModel.MethodInfo methodInfo = new SemanticModel.MethodInfo();
                    methodInfo.name = ctMethod.getSimpleName();
                    methodInfo.signature = ctMethod.getSignature();
                    methodInfo.returnType = ctMethod.getType().getQualifiedName();

                    // Parameters
                    methodInfo.parameters = ctMethod.getParameters().stream()
                        .map(p -> p.getType().getQualifiedName() + " " + p.getSimpleName())
                        .collect(Collectors.toList());

                    // Modifiers
                    methodInfo.isPublic = ctMethod.isPublic();
                    methodInfo.isPrivate = ctMethod.isPrivate();
                    methodInfo.isProtected = ctMethod.isProtected();
                    methodInfo.isStatic = ctMethod.isStatic();
                    methodInfo.isAbstract = ctMethod.isAbstract();

                    // Method body (simplified)
                    if (ctMethod.getBody() != null) {
                        methodInfo.body = ctMethod.getBody().toString();
                    }

                    model.methods.add(methodInfo);
                }

                // Fields
                model.fields = new java.util.ArrayList<>();
                for (var ctField : ctClass.getFields()) {
                    SemanticModel.FieldInfo fieldInfo = new SemanticModel.FieldInfo();
                    fieldInfo.name = ctField.getSimpleName();
                    fieldInfo.type = ctField.getType().getQualifiedName();
                    fieldInfo.isPublic = ctField.isPublic();
                    fieldInfo.isPrivate = ctField.isPrivate();
                    fieldInfo.isProtected = ctField.isProtected();
                    fieldInfo.isStatic = ctField.isStatic();
                    fieldInfo.isFinal = ctField.isFinal();

                    if (ctField.getDefaultExpression() != null) {
                        fieldInfo.initializer = ctField.getDefaultExpression().toString();
                    }

                    model.fields.add(fieldInfo);
                }

                // Annotations
                model.annotations = ctField.getAnnotations().stream()
                    .map(a -> a.getQualifiedName())
                    .collect(Collectors.toList());

                models.add(model);
            }

            // Serialize to JSON
            Gson gson = new GsonBuilder().setPrettyPrinting().create();
            String json = gson.toJson(models);

            // Write to file
            try (FileWriter writer = new FileWriter(outputFile)) {
                writer.write(json);
            }

            System.out.println("Spoon analysis complete. Output written to: " + outputFile);
            System.out.println("Processed " + models.size() + " classes");

        } catch (Exception e) {
            System.err.println("Error during Spoon analysis: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        }
    }
}
