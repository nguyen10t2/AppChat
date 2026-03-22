import { useCallback, useEffect, useRef } from 'react'

type OscType = OscillatorType

const SOUND_LEVEL = {
  ring: 0.014,
  accepted: 0.016,
  ended: 0.018,
} as const

function useSafeAudioContext() {
  const contextRef = useRef<AudioContext | null>(null)

  const getContext = useCallback(() => {
    if (typeof window === 'undefined') return null

    if (!contextRef.current) {
      const Context = window.AudioContext || (window as typeof window & { webkitAudioContext?: typeof AudioContext }).webkitAudioContext
      if (!Context) return null
      contextRef.current = new Context()
    }

    return contextRef.current
  }, [])

  useEffect(() => {
    return () => {
      void contextRef.current?.close().catch(() => undefined)
      contextRef.current = null
    }
  }, [])

  return getContext
}

export function useCallSounds() {
  const ringtoneTimerRef = useRef<number | null>(null)
  const getAudioContext = useSafeAudioContext()

  const playTone = useCallback(
    async (frequency: number, durationMs: number, type: OscType = 'sine', gainValue = 0.03) => {
      const context = getAudioContext()
      if (!context) return

      if (context.state === 'suspended') {
        await context.resume().catch(() => undefined)
      }

      const oscillator = context.createOscillator()
      const gain = context.createGain()

      oscillator.type = type
      oscillator.frequency.value = frequency
      gain.gain.value = gainValue

      oscillator.connect(gain)
      gain.connect(context.destination)

      oscillator.start()
      window.setTimeout(() => {
        oscillator.stop()
        oscillator.disconnect()
        gain.disconnect()
      }, durationMs)
    },
    [getAudioContext],
  )

  const playAccepted = useCallback(() => {
    void playTone(784, 130, 'sine', SOUND_LEVEL.accepted)
    window.setTimeout(() => {
      void playTone(1046, 150, 'sine', SOUND_LEVEL.accepted)
    }, 140)
  }, [playTone])

  const playEnded = useCallback(() => {
    void playTone(420, 190, 'triangle', SOUND_LEVEL.ended)
    window.setTimeout(() => {
      void playTone(300, 230, 'triangle', SOUND_LEVEL.ended)
    }, 200)
  }, [playTone])

  const startRingtone = useCallback(() => {
    if (ringtoneTimerRef.current) return

    const playPattern = () => {
      void playTone(660, 260, 'triangle', SOUND_LEVEL.ring)
      window.setTimeout(() => {
        void playTone(740, 260, 'triangle', SOUND_LEVEL.ring)
      }, 340)
    }

    playPattern()
    ringtoneTimerRef.current = window.setInterval(playPattern, 2200)
  }, [playTone])

  const stopRingtone = useCallback(() => {
    if (!ringtoneTimerRef.current) return
    window.clearInterval(ringtoneTimerRef.current)
    ringtoneTimerRef.current = null
  }, [])

  useEffect(() => {
    return () => {
      stopRingtone()
    }
  }, [stopRingtone])

  return {
    startRingtone,
    stopRingtone,
    playAccepted,
    playEnded,
  }
}
