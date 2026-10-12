package main

import (
	"bytes"
	"encoding/json"
	"log/slog"
	"net/http"
	"os"
	"strings"
)

const MESSAGE = `
🥪 *Hey TEAM_NAME, It's Lunch Time!* 🍽️
`

func main() {
	slog.Info("Starting lunch invitor...")
	url := os.Getenv("GCHAT_URL")
	message := os.Getenv("MESSAGE")
	teamName := os.Getenv("TEAM_NAME")
	if url == "" {
		slog.Error("The environment variable GCHAT_URL is empty")
		os.Exit(1)
	}
	if teamName == "" {
		teamName = "APIM_TEAM"
	}
	if message == "" {
		message = strings.ReplaceAll(MESSAGE, "TEAM_NAME", teamName)
	}

	payload := map[string]string{"text": message}
	jsonPayload, err := json.Marshal(payload)
	if err != nil {
		slog.Error("Error marshaling JSON payload", err)
		return
	}

	resp, err := http.Post(url, "application/json", bytes.NewBuffer(jsonPayload))
	if err != nil {
		slog.Error("Error sending message", err)
		return
	}
	defer resp.Body.Close()

	slog.Info("Message sent successfully", "status", resp.Status)
}
