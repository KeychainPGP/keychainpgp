import java.io.File
import org.apache.tools.ant.taskdefs.condition.Os
import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.logging.LogLevel
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction

open class BuildTask : DefaultTask() {
    @Input
    var rootDirRel: String? = null
    @Input
    var target: String? = null
    @Input
    var release: Boolean? = null

    @TaskAction
    fun assemble() {
        val executable = """npm""";
        try {
            runTauriCli(executable)
        } catch (e: Exception) {
            if (Os.isFamily(Os.FAMILY_WINDOWS)) {
                // Try different Windows-specific extensions
                val fallbacks = listOf(
                    "$executable.exe",
                    "$executable.cmd",
                    "$executable.bat",
                )

                var lastException: Exception = e
                var succeeded = false
                for (fallback in fallbacks) {
                    try {
                        runTauriCli(fallback)
                        succeeded = true
                        break
                    } catch (fallbackException: Exception) {
                        lastException = fallbackException
                    }
                }
                if (!succeeded) throw lastException
            } else {
                throw e;
            }
        }

        // On Windows, Tauri CLI creates symlinks in jniLibs/ that Gradle/AGP
        // may not follow when packaging the APK. Replace them with real copies.
        if (Os.isFamily(Os.FAMILY_WINDOWS)) {
            resolveJniLibSymlinks()
        }
    }

    fun runTauriCli(executable: String) {
        val rootDirRel = rootDirRel ?: throw GradleException("rootDirRel cannot be null")
        val target = target ?: throw GradleException("target cannot be null")
        val release = release ?: throw GradleException("release cannot be null")
        val args = listOf("run", "--", "tauri", "android", "android-studio-script");

        project.exec {
            workingDir(File(project.projectDir, rootDirRel))
            executable(executable)
            args(args)
            if (project.logger.isEnabled(LogLevel.DEBUG)) {
                args("-vv")
            } else if (project.logger.isEnabled(LogLevel.INFO)) {
                args("-v")
            }
            if (release) {
                args("--release")
            }
            args(listOf("--target", target))
        }.assertNormalExitValue()
    }

    private fun resolveJniLibSymlinks() {
        val rootDir = File(project.projectDir, rootDirRel!!).canonicalFile
        // Workspace root is 2 levels up from the Tauri crate root
        val workspaceRoot = rootDir.resolve("../..").canonicalFile
        val profile = if (release!!) "release" else "debug"
        val libName = "libkeychainpgp_ui_lib.so"

        val targetToAbi = mapOf(
            "aarch64" to Pair("arm64-v8a", "aarch64-linux-android"),
            "armv7" to Pair("armeabi-v7a", "armv7-linux-androideabi"),
            "i686" to Pair("x86", "i686-linux-android"),
            "x86_64" to Pair("x86_64", "x86_64-linux-android")
        )

        val (abi, cargoTarget) = targetToAbi[target] ?: return

        val srcFile = File(workspaceRoot, "target/$cargoTarget/$profile/$libName")
        val dstFile = File(project.projectDir, "src/main/jniLibs/$abi/$libName")

        if (srcFile.exists() && srcFile.length() > 0) {
            // Remove symlink or existing file
            dstFile.delete()
            srcFile.copyTo(dstFile, overwrite = true)
            project.logger.lifecycle(
                "Copied native lib: $abi/$libName (${srcFile.length() / 1024} KB)"
            )
        }
    }
}