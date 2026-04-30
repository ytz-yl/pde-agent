import * as React from 'react'
import { cn } from '@/lib/utils'

export const Badge = React.forwardRef<
  HTMLSpanElement,
  React.HTMLAttributes<HTMLSpanElement> & {
    variant?: 'default' | 'secondary' | 'outline' | 'destructive'
  }
>(({ className, variant = 'default', ...props }, ref) => {
  const variantClass = {
    default: 'bg-primary text-primary-foreground',
    secondary: 'bg-secondary text-secondary-foreground',
    outline: 'border border-input text-foreground',
    destructive: 'bg-destructive text-destructive-foreground',
  }[variant]

  return (
    <span
      ref={ref}
      className={cn(
        'inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold transition-colors',
        variantClass,
        className,
      )}
      {...props}
    />
  )
})
Badge.displayName = 'Badge'
