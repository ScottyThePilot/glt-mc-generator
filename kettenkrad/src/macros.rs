macro_rules! all {
  () => (true);
  ($value:expr $(,)?) => ($value);
  ($value1:expr, $($value:expr),+ $(,)?) => ($value1 $(&& $value)*);
}

macro_rules! any {
  () => (false);
  ($value:expr $(,)?) => ($value);
  ($value1:expr, $($value:expr),+ $(,)?) => ($value1 $(|| $value)*);
}

macro_rules! coalesce {
  ($function:expr, $value:expr $(,)?) => {
    $value
  };
  ($function:expr, $value1:expr, $($value:expr),+ $(,)?) => {
    $function($value1, coalesce!($function, $($value),*))
  };
}
