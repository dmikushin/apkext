package main

import (
	"fmt"
	"os"

	"github.com/dmikushin/apkext/internal/config"
	"github.com/dmikushin/apkext/pkg/apk"
	"github.com/spf13/cobra"
)

var (
	version = "dev"
	commit  = "unknown"
	date    = "unknown"
)

func main() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}

var rootCmd = &cobra.Command{
	Use:   "apkext",
	Short: "APK extraction and building tool",
	Long: `APK extraction and building tool with embedded JAR utilities.
Supports unpacking APK files to source code and repacking them back.`,
	Version: fmt.Sprintf("%s (commit: %s, date: %s)", version, commit, date),
}

var unpackCmd = &cobra.Command{
	Use:   "unpack <apk-file>",
	Short: "Unpack APK file to source code",
	Long: `Unpack APK file by extracting resources, converting DEX to JAR,
and decompiling Java classes to source code.`,
	Args: cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		cfg := config.Load()
		extractor := apk.NewExtractor(cfg)
		return extractor.Unpack(args[0])
	},
}

var packCmd = &cobra.Command{
	Use:   "pack <unpacked-dir> <output-apk>",
	Short: "Pack source code back to APK",
	Long: `Pack the unpacked source code directory back into an APK file.`,
	Args: cobra.ExactArgs(2),
	RunE: func(cmd *cobra.Command, args []string) error {
		cfg := config.Load()
		builder := apk.NewBuilder(cfg)
		return builder.Pack(args[0], args[1])
	},
}

func init() {
	// Add commands in desired help order - main commands first
	rootCmd.AddCommand(unpackCmd)
	rootCmd.AddCommand(packCmd)

	// Add explicit help command to make it visible
	helpCmd := &cobra.Command{
		Use:   "help [command]",
		Short: "Help about any command",
		Long: `Help provides help for any command in the application.
Simply type ` + rootCmd.Name() + ` help [path to command] for full details.`,
		Run: func(c *cobra.Command, args []string) {
			if len(args) == 0 {
				rootCmd.Help()
				return
			}
			cmd, _, e := rootCmd.Find(args)
			if cmd == nil || e != nil {
				rootCmd.Printf("Unknown help topic %#q\n", args)
			} else {
				cmd.InitDefaultHelpFlag()
				cmd.Help()
			}
		},
	}
	rootCmd.AddCommand(helpCmd)

	// Customize help template to show main commands first
	rootCmd.SetHelpTemplate(`{{.Short}}{{if .Long}}

{{.Long}}{{end}}{{if .HasExample}}

Examples:
{{.Example}}{{end}}{{if .HasAvailableSubCommands}}

Available Commands:{{range .Commands}}{{if (and .IsAvailableCommand (eq .Name "unpack"))}}
  {{rpad .Name .NamePadding }} {{.Short}}{{end}}{{end}}{{range .Commands}}{{if (and .IsAvailableCommand (eq .Name "pack"))}}
  {{rpad .Name .NamePadding }} {{.Short}}{{end}}{{end}}{{range .Commands}}{{if (and .IsAvailableCommand (not (or (eq .Name "unpack") (eq .Name "pack"))))}}
  {{rpad .Name .NamePadding }} {{.Short}}{{end}}{{end}}{{end}}{{if .HasAvailableLocalFlags}}

Flags:
{{.LocalFlags.FlagUsages | trimTrailingWhitespaces}}{{end}}{{if .HasAvailableInheritedFlags}}

Global Flags:
{{.InheritedFlags.FlagUsages | trimTrailingWhitespaces}}{{end}}{{if .HasHelpSubCommands}}

Additional help topics:{{range .Commands}}{{if .IsAdditionalHelpTopicCommand}}
  {{rpad .Name .NamePadding }} {{.Short}}{{end}}{{end}}{{end}}{{if .HasAvailableSubCommands}}

Use "{{.CommandPath}} [command] --help" for more information about a command.{{end}}
`)
}