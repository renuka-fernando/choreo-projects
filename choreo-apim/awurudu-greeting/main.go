package main

import (
	"bytes"
	"encoding/json"
	"log/slog"
	"net/http"
	"os"
)

const MESSAGE = `
ඔබ සැමට සුබ අලුත් අවුරුද්දක් වේවා !
Wish you all a happy new year !
உங்கள் அனைவருக்கும் இனிய புத்தாண்டு வாழ்த்துக்கள் !
`

func main() {
	slog.Info("Starting new year greeter")
	url := os.Getenv("GCHAT_URL")
	message := os.Getenv("MESSAGE")
	if url == "" {
		slog.Error("The environment variable GCHAT_URL is empty")
		os.Exit(1)
	}
	if message == "" {
		message = MESSAGE
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
