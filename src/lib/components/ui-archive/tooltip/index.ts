import { Tooltip as TooltipPrimitive } from 'bits-ui';
import Trigger from './tooltip-trigger.svelte';
import Content from './tooltip-content.svelte';

const Root = TooltipPrimitive.Root;
const Portal = TooltipPrimitive.Portal;

// Note: Provider should be imported directly from 'bits-ui' where needed
// to avoid $derived rune issues. Example:
// import { Tooltip as TooltipPrimitive } from 'bits-ui';
// <TooltipPrimitive.Provider>...</TooltipPrimitive.Provider>

export {
	Root,
	Trigger,
	Content,
	Portal,
	//
	Root as Tooltip,
	Content as TooltipContent,
	Trigger as TooltipTrigger,
	Portal as TooltipPortal
};
