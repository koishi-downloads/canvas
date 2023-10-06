'use strict'

const bindings = require(globalThis['__prebuilt_canvas'])

module.exports = bindings

bindings.ImageData.prototype.toString = function () {
	return '[object ImageData]'
}

bindings.CanvasGradient.prototype.toString = function () {
	return '[object CanvasGradient]'
}
