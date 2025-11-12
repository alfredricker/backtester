The most fundamental thing to write is the indicators module
that we can build from our data. Since our data is just timestamp OHLCV ticker, our indicators for now are limited to
volume, price, time based (and common ones such as sma which can apply to volume and price)
See types::ohlcv to see the fundamental data

We use the indicators module to build the events module, which tracks mostly 'crossings', i.e., time later than 15:59:59 or average daily volume > 800000, or sma::volume::minutes(50) > ema::volume::hours(5)

Entry types and exit types:
Time based, either specify a timestamp or a time elapse since entry (in trading minutes, hours, or days)
Another option of time based is "auction based" i.e. get  out at 9:30:00 or 16:00:00 
Event based:
price crosses below a threshold (stop loss sell),
indicator crosses below a threshold, etc.
